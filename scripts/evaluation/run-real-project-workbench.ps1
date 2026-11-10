param(
    [string]$ProjectRoot = (Resolve-Path ".").Path,
    [string]$RealProjectPath = (Resolve-Path ".").Path,
    [string]$OutputRoot = "",
    [string]$RrflowPath = "",
    [int]$Port = 9477,
    [string]$RealTestPlanPath = "",
    [string]$ChatExamplesPath = "",
    [string]$ComparatorResultsPath = "",
    [switch]$DevMode,
    [string]$DevModeHistoryPath = "",
    [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-RrflowExecutable {
    param([string]$RequestedPath, [string]$Root)
    if ($RequestedPath -and (Test-Path -LiteralPath $RequestedPath)) {
        return (Resolve-Path -LiteralPath $RequestedPath).Path
    }
    $candidates = @(
        (Join-Path $Root "target\debug\rrflow.exe"),
        (Join-Path $Root "target\release\rrflow.exe")
    )
    foreach ($candidate in $candidates) {
        if (Test-Path -LiteralPath $candidate) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }
    $resolved = Get-Command rrflow -ErrorAction SilentlyContinue
    if ($null -ne $resolved) {
        return $resolved.Source
    }
    throw "Unable to locate rrflow executable. Pass -RrflowPath explicitly."
}

function Invoke-RrflowJson {
    param(
        [string]$Exe,
        [string[]]$Arguments
    )
    $all = @("--json") + $Arguments
    $output = & $Exe @all 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "rrflow failed: $($all -join ' ')`n$output"
    }
    $text = ($output | Out-String).Trim()
    if ([string]::IsNullOrWhiteSpace($text)) {
        throw "rrflow returned empty JSON output for: $($all -join ' ')"
    }
    return $text | ConvertFrom-Json
}

function New-Envelope {
    param(
        [object]$Payload,
        [string]$InstanceId,
        [string]$RequestId,
        [string]$OperationId,
        [object]$IdempotencyKey
    )
    $idempotency = if ($null -eq $IdempotencyKey -or [string]::IsNullOrWhiteSpace([string]$IdempotencyKey)) { $null } else { [string]$IdempotencyKey }
    return @{
        protocol = "rrd"
        protocol_version = 1
        context = @{
            request_id = $RequestId
            operation_id = $OperationId
            idempotency_key = $idempotency
            deadline_unix_ms = $null
        }
        resource = @{
            segments = @(
                @{
                    kind = "instance"
                    id = $InstanceId
                }
            )
        }
        payload = $Payload
    }
}

function Invoke-RrdPost {
    param(
        [string]$BaseUrl,
        [string]$Path,
        [object]$Body,
        [hashtable]$Headers
    )
    $json = $Body | ConvertTo-Json -Depth 100
    try {
        return Invoke-RestMethod -Method Post -Uri ($BaseUrl + $Path) -ContentType "application/json" -Headers $Headers -Body $json
    } catch {
        $responseBody = ""
        if ($_.Exception.Response -and $_.Exception.Response.GetResponseStream) {
            try {
                $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
                $responseBody = $reader.ReadToEnd()
            } catch {
            }
        }
        throw "RRD POST failed at $Path with body: $json`n$($_.Exception.Message)`n$responseBody"
    }
}

function Find-ApiKeyDocument {
    param([string]$ProjectPath)
    $candidates = Get-ChildItem -LiteralPath $ProjectPath -Recurse -Force -File -ErrorAction SilentlyContinue |
        Where-Object { $_.Length -gt 0 -and $_.Length -le 16384 -and $_.Extension -eq ".json" }
    foreach ($candidate in $candidates) {
        try {
            $json = Get-Content -LiteralPath $candidate.FullName -Raw | ConvertFrom-Json
            if ($json.credential -and $json.principal_id -and $json.plan_sha256 -and $json.credential_bytes) {
                return $json
            }
        } catch {
        }
    }
    throw "Unable to find installed API key document under $ProjectPath."
}

function Find-EndpointPath {
    param([object]$Catalogue, [string]$OperationId)
    $entry = $Catalogue.endpoints | Where-Object { $_.operation -eq $OperationId } | Select-Object -First 1
    if ($null -eq $entry) {
        throw "Missing endpoint operation '$OperationId' in catalogue."
    }
    return [string]$entry.path
}

function Write-TextNoBom {
    param([string]$Path, [string]$Content)
    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($Path, $Content, $utf8NoBom)
}

function Read-RealTestPlan {
    param([string]$PlanPath)
    if ([string]::IsNullOrWhiteSpace($PlanPath)) {
        throw "Refusing to run: pass -RealTestPlanPath with a real workload plan. Synthetic smoke is disabled."
    }
    if (-not (Test-Path -LiteralPath $PlanPath)) {
        throw "Real test plan path not found: $PlanPath"
    }
    $resolved = (Resolve-Path -LiteralPath $PlanPath).Path
    $plan = Get-Content -LiteralPath $resolved -Raw | ConvertFrom-Json
    $required = @("plan_version", "name", "workloads", "comparator_baselines", "acceptance")
    foreach ($field in $required) {
        if (-not ($plan.PSObject.Properties.Name -contains $field)) {
            throw "Real test plan is missing required field '$field': $resolved"
        }
    }
    if (@($plan.workloads).Count -lt 3) {
        throw "Real test plan must define at least three workloads: $resolved"
    }
    $expectedIds = @("index_completeness", "retrieval_accuracy", "comparable_latency")
    foreach ($id in $expectedIds) {
        if (-not (@($plan.workloads | Where-Object { $_.id -eq $id }).Count -eq 1)) {
            throw "Real test plan must include workload id '$id': $resolved"
        }
    }
    if (-not ($plan.comparator_baselines.PSObject.Properties.Name -contains "surrealdb")) {
        throw "Real test plan must include comparator baseline 'surrealdb': $resolved"
    }
    if (-not ($plan.comparator_baselines.PSObject.Properties.Name -contains "arangodb")) {
        throw "Real test plan must include comparator baseline 'arangodb': $resolved"
    }
    foreach ($id in $expectedIds) {
        $workload = @($plan.workloads | Where-Object { $_.id -eq $id })[0]
        if ($id -in @("retrieval_accuracy", "comparable_latency")) {
            if (-not ($workload.PSObject.Properties.Name -contains "query") -or [string]::IsNullOrWhiteSpace([string]$workload.query)) {
                throw "Workload '$id' must define a non-empty 'query'."
            }
        }
    }
    if (-not ($plan.acceptance.PSObject.Properties.Name -contains "latency")) {
        throw "Real test plan acceptance must include 'latency' thresholds."
    }
    $latency = $plan.acceptance.latency
    foreach ($field in @("max_ratio_vs_surrealdb_p95", "max_ratio_vs_arangodb_p95")) {
        if (-not ($latency.PSObject.Properties.Name -contains $field)) {
            throw "Real test plan acceptance.latency is missing '$field'."
        }
        if ([double]$latency.$field -le 0) {
            throw "Real test plan acceptance.latency.$field must be > 0."
        }
    }
    return @{
        path = $resolved
        sha256 = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash.ToLowerInvariant()
        plan = $plan
    }
}

function Escape-SingleQuotedLiteral {
    param([string]$Value)
    if ($null -eq $Value) {
        return "''"
    }
    return "'" + ($Value -replace "'", "''") + "'"
}

function Try-GetPromptField {
    param([object]$Object)
    foreach ($name in @("prompt", "user_message", "message", "text", "content", "query")) {
        if (Has-Property -Object $Object -Name $name) {
            $value = [string]$Object.$name
            if (-not [string]::IsNullOrWhiteSpace($value)) {
                return $value.Trim()
            }
        }
    }
    return $null
}

function Read-ChatExamples {
    param([string]$Path, [int]$MinimumCount)
    if ([string]::IsNullOrWhiteSpace($Path)) {
        throw "Refusing to run: pass -ChatExamplesPath with your real chat examples."
    }
    if (-not (Test-Path -LiteralPath $Path)) {
        throw "Chat examples path not found: $Path"
    }
    $resolved = (Resolve-Path -LiteralPath $Path).Path
    $raw = Get-Content -LiteralPath $resolved -Raw
    $parsed = $raw | ConvertFrom-Json
    $records = @()
    if ($parsed -is [System.Array]) {
        $records = @($parsed)
    } elseif (Has-Property -Object $parsed -Name "prompts") {
        $records = @($parsed.prompts)
    } elseif (Has-Property -Object $parsed -Name "examples") {
        $records = @($parsed.examples)
    } else {
        $records = @($parsed)
    }
    $prompts = New-Object System.Collections.Generic.List[string]
    foreach ($record in $records) {
        if ($record -is [string]) {
            $candidate = $record.Trim()
            if (-not [string]::IsNullOrWhiteSpace($candidate)) {
                $prompts.Add($candidate)
            }
            continue
        }
        $candidateObjectPrompt = Try-GetPromptField -Object $record
        if (-not [string]::IsNullOrWhiteSpace($candidateObjectPrompt)) {
            $prompts.Add($candidateObjectPrompt)
        }
    }
    $unique = @($prompts | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -Unique)
    if ($unique.Count -lt $MinimumCount) {
        throw "Chat examples file must provide at least $MinimumCount unique prompts: $resolved"
    }
    $hashInput = ($unique | ForEach-Object { $_.Trim() }) -join "`n"
    $sha = [System.BitConverter]::ToString(
        [System.Security.Cryptography.SHA256]::Create().ComputeHash([System.Text.Encoding]::UTF8.GetBytes($hashInput))
    ).Replace("-", "").ToLowerInvariant()
    return @{
        path = $resolved
        prompts = $unique
        sha256 = $sha
    }
}

function Has-Property {
    param([object]$Object, [string]$Name)
    if ($null -eq $Object) {
        return $false
    }
    return $Object.PSObject.Properties.Name -contains $Name
}

function Get-WorkloadById {
    param([object]$Plan, [string]$WorkloadId)
    $matches = @($Plan.workloads | Where-Object { $_.id -eq $WorkloadId })
    if ($matches.Count -ne 1) {
        throw "Real test plan must include exactly one workload '$WorkloadId'."
    }
    return $matches[0]
}

function Resolve-OutcomePayload {
    param([object]$Envelope, [string]$OperationName)
    if (-not (Has-Property -Object $Envelope -Name "outcome")) {
        throw "Operation '$OperationName' response is missing outcome."
    }
    if ($Envelope.outcome.status -eq "ok") {
        return $Envelope.outcome.payload
    }
    $message = if (Has-Property -Object $Envelope.outcome -Name "error") {
        $Envelope.outcome.error.message
    } else {
        "unknown error"
    }
    throw "Operation '$OperationName' failed: $message"
}

function Resolve-QueryText {
    param(
        [string]$Template,
        [uint64]$KnownCursor,
        [string]$PromptText
    )
    $now = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    $prompt = if ($null -eq $PromptText) { "" } else { $PromptText }
    $promptSql = Escape-SingleQuotedLiteral -Value $prompt
    return $Template.
        Replace("{{KNOWN_CURSOR}}", [string]$KnownCursor).
        Replace("{{VALID_NOW_MS}}", [string]$now).
        Replace("{{PROMPT}}", $prompt).
        Replace("{{PROMPT_SQL}}", $promptSql)
}

function Invoke-QueryWorkload {
    param(
        [string]$BaseUrl,
        [string]$Path,
        [hashtable]$SessionHeaders,
        [string]$InstanceId,
        [string]$Scope,
        [string]$QueryText,
        [object]$Budget,
        [string]$RequestPrefix
    )
    $body = New-Envelope -Payload @{
        scope = $Scope
        query = $QueryText
        parameters = @{}
        budget = $Budget
    } -InstanceId $InstanceId -RequestId "$RequestPrefix-request" -OperationId "$RequestPrefix-operation" -IdempotencyKey $null
    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
    $response = Invoke-RrdPost -BaseUrl $BaseUrl -Path $Path -Body $body -Headers $SessionHeaders
    $stopwatch.Stop()
    $payload = Resolve-OutcomePayload -Envelope $response -OperationName $RequestPrefix
    return @{
        latency_ms = [double]$stopwatch.Elapsed.TotalMilliseconds
        payload = $payload
    }
}

function Get-Percentile {
    param([double[]]$Values, [double]$Percentile)
    if ($Values.Count -eq 0) {
        throw "Cannot compute percentile of empty sequence."
    }
    $sorted = $Values | Sort-Object
    if ($sorted.Count -eq 1) {
        return [double]$sorted[0]
    }
    $rank = ($Percentile / 100.0) * ($sorted.Count - 1)
    $lower = [int][Math]::Floor($rank)
    $upper = [int][Math]::Ceiling($rank)
    if ($lower -eq $upper) {
        return [double]$sorted[$lower]
    }
    $weight = $rank - $lower
    return ([double]$sorted[$lower] * (1.0 - $weight)) + ([double]$sorted[$upper] * $weight)
}

function Get-ComparatorMetric {
    param(
        [object]$Comparator,
        [string]$WorkloadId,
        [string]$Provider,
        [string]$Metric
    )
    if (-not (Has-Property -Object $Comparator -Name "workloads")) {
        throw "Comparator results are missing top-level 'workloads'."
    }
    $workloads = $Comparator.workloads
    if (-not (Has-Property -Object $workloads -Name $WorkloadId)) {
        throw "Comparator results are missing workload '$WorkloadId'."
    }
    $entry = $workloads.$WorkloadId
    if (-not (Has-Property -Object $entry -Name $Provider)) {
        throw "Comparator workload '$WorkloadId' is missing provider '$Provider'."
    }
    $providerEntry = $entry.$Provider
    if (-not (Has-Property -Object $providerEntry -Name $Metric)) {
        throw "Comparator workload '$WorkloadId' provider '$Provider' is missing metric '$Metric'."
    }
    return [double]$providerEntry.$Metric
}

function Resolve-DevModeHistoryPath {
    param(
        [bool]$Enabled,
        [string]$RequestedPath,
        [string]$ProjectRoot,
        [string]$OutputBase
    )
    if (-not $Enabled) {
        return $null
    }
    if ([string]::IsNullOrWhiteSpace($RequestedPath)) {
        return (Join-Path $OutputBase "workbench-dev-history.jsonl")
    }
    if ([System.IO.Path]::IsPathRooted($RequestedPath)) {
        return $RequestedPath
    }
    return (Join-Path $ProjectRoot $RequestedPath)
}

function Write-JsonLine {
    param(
        [string]$Path,
        [object]$Value
    )
    $directory = Split-Path -Parent $Path
    if (-not [string]::IsNullOrWhiteSpace($directory) -and -not (Test-Path -LiteralPath $directory)) {
        New-Item -ItemType Directory -Path $directory -Force | Out-Null
    }
    $line = $Value | ConvertTo-Json -Depth 100 -Compress
    Add-Content -LiteralPath $Path -Value $line -Encoding UTF8
}

function Read-JsonLines {
    param([string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) {
        return @()
    }
    $rows = New-Object System.Collections.Generic.List[object]
    foreach ($line in (Get-Content -LiteralPath $Path)) {
        if ([string]::IsNullOrWhiteSpace($line)) {
            continue
        }
        try {
            $rows.Add(($line | ConvertFrom-Json))
        } catch {
            throw "Dev mode history contains invalid JSON line in $Path"
        }
    }
    return @($rows)
}

function New-ExecutionManifest {
    param(
        [object]$PlanRecord,
        [object]$ChatRecord,
        [string]$ComparatorPath,
        [bool]$DryRunMode,
        [bool]$DevModeEnabled
    )
    $comparatorPathValue = if ($null -eq $ComparatorPath) { "" } else { [string]$ComparatorPath }
    $comparatorPathOutput = $null
    if (-not [string]::IsNullOrWhiteSpace($comparatorPathValue)) {
        $comparatorPathOutput = $comparatorPathValue
    }
    return [ordered]@{
        generated_at = (Get-Date).ToString("o")
        dry_run = $DryRunMode
        dev_mode = $DevModeEnabled
        plan = [ordered]@{
            path = $PlanRecord.path
            sha256 = $PlanRecord.sha256
            name = $PlanRecord.plan.name
            plan_version = $PlanRecord.plan.plan_version
            acceptance = $PlanRecord.plan.acceptance
            comparator_baselines = $PlanRecord.plan.comparator_baselines
        }
        chat_examples = [ordered]@{
            path = $ChatRecord.path
            prompt_count = @($ChatRecord.prompts).Count
            prompt_set_sha256 = $ChatRecord.sha256
        }
        comparator_results_path = $comparatorPathOutput
        workloads = $PlanRecord.plan.workloads
    }
}

$projectRootResolved = (Resolve-Path -LiteralPath $ProjectRoot).Path
$realProjectResolved = (Resolve-Path -LiteralPath $RealProjectPath).Path
$rrflowExe = Get-RrflowExecutable -RequestedPath $RrflowPath -Root $projectRootResolved
$realTestPlan = Read-RealTestPlan -PlanPath $RealTestPlanPath
$minimumPrompts = if (Has-Property -Object $realTestPlan.plan.acceptance -Name "min_prompt_examples") {
    [int]$realTestPlan.plan.acceptance.min_prompt_examples
} else {
    3
}
$chatExamples = Read-ChatExamples -Path $ChatExamplesPath -MinimumCount $minimumPrompts
$outputBase = if ($OutputRoot) { $OutputRoot } else { Join-Path $projectRootResolved "eval\results" }
$devModeHistoryResolved = Resolve-DevModeHistoryPath -Enabled ([bool]$DevMode) -RequestedPath $DevModeHistoryPath -ProjectRoot $projectRootResolved -OutputBase $outputBase
$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$runRoot = Join-Path $outputBase "real-project-workbench-$timestamp"
New-Item -ItemType Directory -Path $runRoot -Force | Out-Null
$rrflowRunner = Join-Path $runRoot "rrflow-runner.exe"
Copy-Item -LiteralPath $rrflowExe -Destination $rrflowRunner -Force
$reportPath = Join-Path $runRoot "workbench-report.json"
$promptSetPath = Join-Path $runRoot "prompt-set.json"
Write-TextNoBom -Path $promptSetPath -Content (@($chatExamples.prompts) | ConvertTo-Json -Depth 20)
$executionManifestPath = Join-Path $runRoot "execution-manifest.json"
$executionManifest = New-ExecutionManifest -PlanRecord $realTestPlan -ChatRecord $chatExamples -ComparatorPath $ComparatorResultsPath -DryRunMode ([bool]$DryRun) -DevModeEnabled ([bool]$DevMode)
Write-TextNoBom -Path $executionManifestPath -Content ($executionManifest | ConvertTo-Json -Depth 100)
Write-Host "Execution manifest: $executionManifestPath"
Write-Host "Prompt set artifact: $promptSetPath"
if ($DevMode) {
    Write-Host "Dev mode history: $devModeHistoryResolved"
}

$report = [ordered]@{
    generated_at = (Get-Date).ToString("o")
    project_root = $projectRootResolved
    real_project = $realProjectResolved
    rrflow_executable = $rrflowExe
    rrflow_runner = $rrflowRunner
    run_root = $runRoot
    dry_run = [bool]$DryRun
    real_test_plan = [ordered]@{
        path = $realTestPlan.path
        sha256 = $realTestPlan.sha256
        name = $realTestPlan.plan.name
        plan_version = $realTestPlan.plan.plan_version
    }
    chat_examples = [ordered]@{
        path = $chatExamples.path
        prompt_count = @($chatExamples.prompts).Count
        prompt_set_sha256 = $chatExamples.sha256
    }
    artifacts = [ordered]@{
        execution_manifest = $executionManifestPath
        prompt_set = $promptSetPath
    }
    dev_mode = [ordered]@{
        enabled = [bool]$DevMode
        history_path = $devModeHistoryResolved
        baseline = $null
        trend = $null
        summary_artifact = $null
    }
    installation = $null
    connectome_view = $null
    workloads = [ordered]@{}
    comparison = $null
}

if ($DryRun) {
    $report.installation = @{
        status = "skipped"
        reason = "dry-run"
    }
    Write-TextNoBom -Path $reportPath -Content ($report | ConvertTo-Json -Depth 100)
    Write-Host "Dry run complete: $reportPath"
    exit 0
}

if ([string]::IsNullOrWhiteSpace($ComparatorResultsPath)) {
    throw "Refusing to run: pass -ComparatorResultsPath with measured SurrealDB/ArangoDB baseline results for the same workloads."
}
if (-not (Test-Path -LiteralPath $ComparatorResultsPath)) {
    throw "Comparator results path not found: $ComparatorResultsPath"
}
$comparatorResolved = (Resolve-Path -LiteralPath $ComparatorResultsPath).Path
$comparatorResults = Get-Content -LiteralPath $comparatorResolved -Raw | ConvertFrom-Json
if (-not (Has-Property -Object $comparatorResults -Name "prompt_set_sha256")) {
    throw "Comparator results must include prompt_set_sha256 for chat-derived parity checks."
}
if ([string]$comparatorResults.prompt_set_sha256 -ne [string]$chatExamples.sha256) {
    throw "Comparator prompt_set_sha256 does not match extracted chat prompt set."
}

$serveProcess = $null
$sessionId = $null
$sessionHeaders = $null
try {
    $identity = Invoke-RrflowJson -Exe $rrflowRunner -Arguments @("install", "identity")
    $installationIdentity = if ($identity.installation_identity) {
        [string]$identity.installation_identity
    } elseif ($identity.identity) {
        [string]$identity.identity
    } else {
        throw "install identity output does not include installation_identity."
    }

    $plan = Invoke-RrflowJson -Exe $rrflowRunner -Arguments @(
        "install", "plan",
        "--project", $realProjectResolved,
        "--mode", "existing",
        "--identity", $installationIdentity
    )
    $planPath = Join-Path $runRoot "installation-plan.json"
    Write-TextNoBom -Path $planPath -Content ($plan | ConvertTo-Json -Depth 100)
    $apply = Invoke-RrflowJson -Exe $rrflowRunner -Arguments @(
        "install", "apply",
        "--project", $realProjectResolved,
        "--mode", "existing",
        "--plan", $planPath,
        "--expect", [string]$plan.installation.plan_sha256
    )

    $report.installation = [ordered]@{
        identity = if ($identity.PSObject.Properties.Name -contains "identity") { $identity.identity } else { $null }
        installation_identity = $installationIdentity
        plan_sha256 = $plan.installation.plan_sha256
        source_sha256 = $plan.attunement.source_sha256
        applied_runtime_cursor = $apply.runtime_cursor
        control_journal_sequence = $apply.control_journal_sequence
        idempotent_replay = $apply.idempotent_replay
    }
    $verify = Invoke-RrflowJson -Exe $rrflowRunner -Arguments @("verify", "--project", $realProjectResolved, "--level", "quick")

    $serveOut = Join-Path $runRoot "serve.stdout.log"
    $serveErr = Join-Path $runRoot "serve.stderr.log"
    $serveProcess = Start-Process -FilePath $rrflowRunner -ArgumentList @(
        "serve",
        "--project", $realProjectResolved,
        "--bind", "127.0.0.1:$Port"
    ) -PassThru -NoNewWindow -RedirectStandardOutput $serveOut -RedirectStandardError $serveErr

    $baseUrl = "http://127.0.0.1:$Port"
    $readyData = $null
    $attempt = 0
    while ($attempt -lt 80) {
        Start-Sleep -Milliseconds 250
        try {
            $readyData = Invoke-RestMethod -Uri "$baseUrl/v1/health/ready" -Method Get
            if ($null -ne $readyData.runtime_cursor) {
                break
            }
        } catch {
        }
        $attempt += 1
    }
    if ($null -eq $readyData) {
        throw "RRFlow service failed to become ready on $baseUrl."
    }

    $ready = Invoke-RrflowJson -Exe $rrflowRunner -Arguments @("ready", "--project", $realProjectResolved, "--address", "127.0.0.1:$Port")
    $readyPath = Join-Path $runRoot "connectome-ready.json"
    Write-TextNoBom -Path $readyPath -Content ($ready | ConvertTo-Json -Depth 100)
    $capabilitiesPath = Join-Path $runRoot "connectome-capabilities.json"
    Write-TextNoBom -Path $capabilitiesPath -Content ($ready.capabilities | ConvertTo-Json -Depth 100)

    $instanceId = [string]$ready.capabilities.instance.id
    $report.connectome_view = [ordered]@{
        ready_runtime_cursor = $ready.readiness.runtime_cursor
        capability_count = @($ready.capabilities.capabilities).Count
        instance_id = $instanceId
        endpoint_catalogue_sha256 = if ($ready.endpoint_catalogue.PSObject.Properties.Name -contains "sha256") { $ready.endpoint_catalogue.sha256 } else { $null }
        openapi_sha256 = $ready.openapi_sha256
        artifacts = @{
            ready = $readyPath
            capabilities = $capabilitiesPath
        }
    }

    $catalogue = $ready.endpoint_catalogue
    $instanceScope = "instance:$instanceId"
    $queryPath = Find-EndpointPath -Catalogue $catalogue -OperationId "query-execute"

    $apiDoc = Find-ApiKeyDocument -ProjectPath $realProjectResolved
    $createHeaders = @{
        "X-RRD-Principal" = [string]$apiDoc.principal_id
        "Authorization" = "ApiKey $($apiDoc.credential)"
    }
    $sessionCreate = New-Envelope -Payload @{
        limits = @{
            idle_timeout_ms = 60000
            absolute_timeout_ms = 300000
            max_open_transactions = 1
        }
    } -InstanceId $instanceId -RequestId "workbench-session-create" -OperationId "workbench-session-create" -IdempotencyKey "workbench-session-create"
    $sessionResponse = Invoke-RrdPost -BaseUrl $baseUrl -Path "/v1/sessions" -Body $sessionCreate -Headers $createHeaders
    $sessionPayload = Resolve-OutcomePayload -Envelope $sessionResponse -OperationName "session-create"
    $sessionId = [string]$sessionPayload.session_id
    $sessionToken = [string]$sessionPayload.token
    $sessionHeaders = @{
        "X-RRD-Session" = $sessionId
        "Authorization" = "Bearer $sessionToken"
    }

    $indexWorkload = Get-WorkloadById -Plan $realTestPlan.plan -WorkloadId "index_completeness"
    $indexRequireSourceCurrent = if (Has-Property -Object $indexWorkload -Name "require_source_current") { [bool]$indexWorkload.require_source_current } else { $true }
    $indexMinPassRatio = if (Has-Property -Object $indexWorkload -Name "min_check_pass_ratio") { [double]$indexWorkload.min_check_pass_ratio } else { 1.0 }
    $indexPassRatio = if (@($verify.checks).Count -eq 0) { 0.0 } else { [double](@($verify.checks | Where-Object { $_.passed -eq $true }).Count) / [double]@($verify.checks).Count }
    $indexPassed = ($indexPassRatio -ge $indexMinPassRatio) -and ((-not $indexRequireSourceCurrent) -or [bool]$verify.attunement_source_current)

    $report.workloads.index_completeness = [ordered]@{
        workload_id = "index_completeness"
        verification_status = $verify.status
        attunement_source_current = $verify.attunement_source_current
        runtime_cursor = $verify.runtime_cursor
        checks_passed = @($verify.checks | Where-Object { $_.passed -eq $true }).Count
        checks_total = @($verify.checks).Count
        check_pass_ratio = $indexPassRatio
        required_min_check_pass_ratio = $indexMinPassRatio
        passed = $indexPassed
        verification_plan_sha256 = $verify.plan_sha256
    }

    $retrievalWorkload = Get-WorkloadById -Plan $realTestPlan.plan -WorkloadId "retrieval_accuracy"
    if (-not (Has-Property -Object $retrievalWorkload -Name "query")) {
        throw "retrieval_accuracy workload must define 'query'."
    }
    $retrievalScope = if (Has-Property -Object $retrievalWorkload -Name "scope") { [string]$retrievalWorkload.scope } else { $instanceScope }
    $defaultBudget = @{
        max_storage_keys = 5000
        max_input_rows = 5000
        max_rows = 100
        max_output_bytes = 65536
        max_batch_rows = 64
        max_memory_bytes = 67108864
        max_spill_bytes = 268435456
        max_elapsed_ms = 30000
    }
    $retrievalBudget = if (Has-Property -Object $retrievalWorkload -Name "budget") { $retrievalWorkload.budget } else { $defaultBudget }
    $retrievalPromptLimit = if (Has-Property -Object $retrievalWorkload -Name "prompt_count") { [int]$retrievalWorkload.prompt_count } else { [int]@($chatExamples.prompts).Count }
    if ($retrievalPromptLimit -lt 1) {
        throw "retrieval_accuracy prompt_count must be at least 1."
    }
    $retrievalPrompts = @($chatExamples.prompts | Select-Object -First $retrievalPromptLimit)
    $retrievalCases = New-Object System.Collections.Generic.List[object]
    foreach ($i in 0..($retrievalPrompts.Count - 1)) {
        $prompt = [string]$retrievalPrompts[$i]
        $retrievalQuery = Resolve-QueryText -Template ([string]$retrievalWorkload.query) -KnownCursor ([uint64]$ready.readiness.runtime_cursor) -PromptText $prompt
        $retrievalResult = Invoke-QueryWorkload -BaseUrl $baseUrl -Path $queryPath -SessionHeaders $sessionHeaders -InstanceId $instanceId -Scope $retrievalScope -QueryText $retrievalQuery -Budget $retrievalBudget -RequestPrefix ("retrieval-accuracy-{0}" -f $i)
        $retrievalPayload = $retrievalResult.payload
        $retrievalCases.Add([ordered]@{
            index = $i
            prompt = $prompt
            query = $retrievalQuery
            latency_ms = $retrievalResult.latency_ms
            returned_rows = [uint64]$retrievalPayload.execution.returned_rows
            truncated = [bool]$retrievalPayload.execution.truncated
            read_manifest_sha256 = $retrievalPayload.read_manifest_sha256
        })
    }
    $retrievalRowsArray = @($retrievalCases | ForEach-Object { [uint64]$_.returned_rows })
    $retrievalLatencyArray = @($retrievalCases | ForEach-Object { [double]$_.latency_ms })
    $retrievalTruncatedAny = @($retrievalCases | Where-Object { $_.truncated -eq $true }).Count -gt 0
    $retrievalMinRows = if (Has-Property -Object $retrievalWorkload -Name "min_rows") { [uint64]$retrievalWorkload.min_rows } else { 1 }
    $retrievalMaxRows = if (Has-Property -Object $retrievalWorkload -Name "max_rows") { [uint64]$retrievalWorkload.max_rows } else { [uint64]18446744073709551615 }
    $retrievalRequireUntruncated = if (Has-Property -Object $retrievalWorkload -Name "require_untruncated") { [bool]$retrievalWorkload.require_untruncated } else { $true }
    $retrievalMinSuccessRatio = if (Has-Property -Object $retrievalWorkload -Name "min_success_ratio") { [double]$retrievalWorkload.min_success_ratio } else { 1.0 }
    $retrievalPassingCases = @($retrievalCases | Where-Object {
            ([uint64]$_.returned_rows -ge $retrievalMinRows) -and ([uint64]$_.returned_rows -le $retrievalMaxRows) -and ((-not $retrievalRequireUntruncated) -or (-not [bool]$_.truncated))
        }).Count
    $retrievalCaseCount = @($retrievalCases).Count
    $retrievalSuccessRatio = if ($retrievalCaseCount -eq 0) { 0.0 } else { [double]$retrievalPassingCases / [double]$retrievalCaseCount }
    $retrievalPassed = $retrievalSuccessRatio -ge $retrievalMinSuccessRatio
    Write-TextNoBom -Path (Join-Path $runRoot "retrieval-query-results.json") -Content (@($retrievalCases) | ConvertTo-Json -Depth 100)

    $report.workloads.retrieval_accuracy = [ordered]@{
        workload_id = "retrieval_accuracy"
        prompt_count = $retrievalCaseCount
        scope = $retrievalScope
        min_latency_ms = if ($retrievalLatencyArray.Count -eq 0) { $null } else { ($retrievalLatencyArray | Measure-Object -Minimum).Minimum }
        max_latency_ms = if ($retrievalLatencyArray.Count -eq 0) { $null } else { ($retrievalLatencyArray | Measure-Object -Maximum).Maximum }
        p95_latency_ms = if ($retrievalLatencyArray.Count -eq 0) { $null } else { Get-Percentile -Values $retrievalLatencyArray -Percentile 95 }
        returned_rows_min = if ($retrievalRowsArray.Count -eq 0) { $null } else { ($retrievalRowsArray | Measure-Object -Minimum).Minimum }
        returned_rows_max = if ($retrievalRowsArray.Count -eq 0) { $null } else { ($retrievalRowsArray | Measure-Object -Maximum).Maximum }
        truncated_any = $retrievalTruncatedAny
        min_rows = $retrievalMinRows
        max_rows = $retrievalMaxRows
        require_untruncated = $retrievalRequireUntruncated
        min_success_ratio = $retrievalMinSuccessRatio
        success_ratio = $retrievalSuccessRatio
        passing_cases = $retrievalPassingCases
        total_cases = $retrievalCaseCount
        passed = $retrievalPassed
    }

    $latencyWorkload = Get-WorkloadById -Plan $realTestPlan.plan -WorkloadId "comparable_latency"
    if (-not (Has-Property -Object $latencyWorkload -Name "query")) {
        throw "comparable_latency workload must define 'query'."
    }
    $latencyScope = if (Has-Property -Object $latencyWorkload -Name "scope") { [string]$latencyWorkload.scope } else { $instanceScope }
    $latencyBudget = if (Has-Property -Object $latencyWorkload -Name "budget") { $latencyWorkload.budget } else { $defaultBudget }
    $warmup = if (Has-Property -Object $latencyWorkload -Name "warmup_iterations") { [int]$latencyWorkload.warmup_iterations } else { 1 }
    $iterations = if (Has-Property -Object $latencyWorkload -Name "iterations") { [int]$latencyWorkload.iterations } else { 5 }
    $latencyPromptLimit = if (Has-Property -Object $latencyWorkload -Name "prompt_count") { [int]$latencyWorkload.prompt_count } else { [int]@($chatExamples.prompts).Count }
    if ($latencyPromptLimit -lt 1) {
        throw "comparable_latency prompt_count must be at least 1."
    }
    $latencyPrompts = @($chatExamples.prompts | Select-Object -First $latencyPromptLimit)
    if ($iterations -lt 3) {
        throw "comparable_latency workload iterations must be at least 3."
    }
    for ($i = 0; $i -lt $warmup; $i++) {
        $prompt = [string]$latencyPrompts[$i % $latencyPrompts.Count]
        $latencyQuery = Resolve-QueryText -Template ([string]$latencyWorkload.query) -KnownCursor ([uint64]$ready.readiness.runtime_cursor) -PromptText $prompt
        $null = Invoke-QueryWorkload -BaseUrl $baseUrl -Path $queryPath -SessionHeaders $sessionHeaders -InstanceId $instanceId -Scope $latencyScope -QueryText $latencyQuery -Budget $latencyBudget -RequestPrefix ("latency-warmup-{0}" -f $i)
    }
    $samples = New-Object System.Collections.Generic.List[double]
    $latencySampleDetails = New-Object System.Collections.Generic.List[object]
    for ($i = 0; $i -lt $iterations; $i++) {
        $prompt = [string]$latencyPrompts[$i % $latencyPrompts.Count]
        $latencyQuery = Resolve-QueryText -Template ([string]$latencyWorkload.query) -KnownCursor ([uint64]$ready.readiness.runtime_cursor) -PromptText $prompt
        $result = Invoke-QueryWorkload -BaseUrl $baseUrl -Path $queryPath -SessionHeaders $sessionHeaders -InstanceId $instanceId -Scope $latencyScope -QueryText $latencyQuery -Budget $latencyBudget -RequestPrefix ("latency-run-{0}" -f $i)
        $samples.Add([double]$result.latency_ms)
        $latencySampleDetails.Add([ordered]@{
            iteration = $i
            prompt = $prompt
            query = $latencyQuery
            latency_ms = [double]$result.latency_ms
            returned_rows = [uint64]$result.payload.execution.returned_rows
            truncated = [bool]$result.payload.execution.truncated
        })
    }
    $sampleArray = @($samples.ToArray())
    $latencyP50 = Get-Percentile -Values $sampleArray -Percentile 50
    $latencyP95 = Get-Percentile -Values $sampleArray -Percentile 95
    $latencyP99 = Get-Percentile -Values $sampleArray -Percentile 99

    $surrealP95 = Get-ComparatorMetric -Comparator $comparatorResults -WorkloadId "comparable_latency" -Provider "surrealdb" -Metric "p95_ms"
    $arangoP95 = Get-ComparatorMetric -Comparator $comparatorResults -WorkloadId "comparable_latency" -Provider "arangodb" -Metric "p95_ms"
    if ($surrealP95 -le 0 -or $arangoP95 -le 0) {
        throw "Comparator p95 metrics must be > 0 for comparable_latency."
    }
    $ratioSurreal = $latencyP95 / $surrealP95
    $ratioArango = $latencyP95 / $arangoP95
    $maxRatioSurreal = [double]$realTestPlan.plan.acceptance.latency.max_ratio_vs_surrealdb_p95
    $maxRatioArango = [double]$realTestPlan.plan.acceptance.latency.max_ratio_vs_arangodb_p95
    $latencyPassed = ($ratioSurreal -le $maxRatioSurreal) -and ($ratioArango -le $maxRatioArango)

    $report.workloads.comparable_latency = [ordered]@{
        workload_id = "comparable_latency"
        query_template = [string]$latencyWorkload.query
        scope = $latencyScope
        prompt_count = @($latencyPrompts).Count
        warmup_iterations = $warmup
        iterations = $iterations
        samples_ms = $sampleArray
        sample_details = @($latencySampleDetails)
        p50_ms = $latencyP50
        p95_ms = $latencyP95
        p99_ms = $latencyP99
        baseline_surrealdb_p95_ms = $surrealP95
        baseline_arangodb_p95_ms = $arangoP95
        ratio_vs_surrealdb_p95 = $ratioSurreal
        ratio_vs_arangodb_p95 = $ratioArango
        max_ratio_vs_surrealdb_p95 = $maxRatioSurreal
        max_ratio_vs_arangodb_p95 = $maxRatioArango
        passed = $latencyPassed
    }

    $requireComparable = if (Has-Property -Object $realTestPlan.plan.acceptance -Name "require_comparable") { [bool]$realTestPlan.plan.acceptance.require_comparable } else { $true }
    $overallComparable = $indexPassed -and $retrievalPassed -and $latencyPassed

    $comparison = [ordered]@{
        surrealdb = @{
            comparable_latency_p95_ms = $surrealP95
            rrflow_p95_ratio = $ratioSurreal
        }
        arangodb = @{
            comparable_latency_p95_ms = $arangoP95
            rrflow_p95_ratio = $ratioArango
        }
        require_comparable = $requireComparable
        comparable = if ($requireComparable) { $overallComparable } else { $true }
        reason = if ($overallComparable) { "All required workloads passed and comparator latency ratios are within plan thresholds." } else { "One or more required workloads or comparator thresholds failed." }
    }
    $comparison.external_artifact = $comparatorResolved
    $comparison.plan_name = $realTestPlan.plan.name
    $comparison.plan_sha256 = $realTestPlan.sha256
    $comparison.prompt_set_sha256 = $chatExamples.sha256
    $report.comparison = $comparison

} finally {
    if ($null -ne $sessionHeaders -and $null -ne $sessionId -and $null -ne $serveProcess -and -not $serveProcess.HasExited) {
        try {
            $closeBody = New-Envelope -Payload @{} -InstanceId "instance-close" -RequestId "workbench-session-close" -OperationId "workbench-session-close" -IdempotencyKey $null
            $null = Invoke-RestMethod -Method Delete -Uri ("http://127.0.0.1:$Port/v1/sessions/$sessionId") -ContentType "application/json" -Headers $sessionHeaders -Body ($closeBody | ConvertTo-Json -Depth 20)
        } catch {
        }
    }
    if ($null -ne $serveProcess -and -not $serveProcess.HasExited) {
        Stop-Process -Id $serveProcess.Id
    }
}

if ($DevMode -and -not $DryRun) {
    $devSummaryPath = Join-Path $runRoot "dev-mode-summary.json"
    $currentSummary = [ordered]@{
        recorded_at = (Get-Date).ToString("o")
        run_root = $runRoot
        report_path = $reportPath
        plan_sha256 = $realTestPlan.sha256
        prompt_set_sha256 = $chatExamples.sha256
        index_passed = [bool]$report.workloads.index_completeness.passed
        retrieval_success_ratio = [double]$report.workloads.retrieval_accuracy.success_ratio
        retrieval_p95_ms = [double]$report.workloads.retrieval_accuracy.p95_latency_ms
        latency_p95_ms = [double]$report.workloads.comparable_latency.p95_ms
        ratio_vs_surrealdb_p95 = [double]$report.workloads.comparable_latency.ratio_vs_surrealdb_p95
        ratio_vs_arangodb_p95 = [double]$report.workloads.comparable_latency.ratio_vs_arangodb_p95
        comparable = [bool]$report.comparison.comparable
    }
    $historyRows = Read-JsonLines -Path $devModeHistoryResolved
    $matchingRows = @($historyRows | Where-Object {
            (Has-Property -Object $_ -Name "plan_sha256") -and
            (Has-Property -Object $_ -Name "prompt_set_sha256") -and
            ([string]$_.plan_sha256 -eq [string]$currentSummary.plan_sha256) -and
            ([string]$_.prompt_set_sha256 -eq [string]$currentSummary.prompt_set_sha256)
        })
    if ($matchingRows.Count -gt 0) {
        $baseline = $matchingRows[$matchingRows.Count - 1]
        $latencyDelta = [double]$currentSummary.latency_p95_ms - [double]$baseline.latency_p95_ms
        $retrievalDelta = [double]$currentSummary.retrieval_p95_ms - [double]$baseline.retrieval_p95_ms
        $surrealRatioDelta = [double]$currentSummary.ratio_vs_surrealdb_p95 - [double]$baseline.ratio_vs_surrealdb_p95
        $arangoRatioDelta = [double]$currentSummary.ratio_vs_arangodb_p95 - [double]$baseline.ratio_vs_arangodb_p95
        $report.dev_mode.baseline = [ordered]@{
            recorded_at = $baseline.recorded_at
            run_root = $baseline.run_root
            latency_p95_ms = $baseline.latency_p95_ms
            retrieval_p95_ms = $baseline.retrieval_p95_ms
            ratio_vs_surrealdb_p95 = $baseline.ratio_vs_surrealdb_p95
            ratio_vs_arangodb_p95 = $baseline.ratio_vs_arangodb_p95
            comparable = $baseline.comparable
        }
        $report.dev_mode.trend = [ordered]@{
            latency_p95_delta_ms = $latencyDelta
            retrieval_p95_delta_ms = $retrievalDelta
            ratio_vs_surrealdb_p95_delta = $surrealRatioDelta
            ratio_vs_arangodb_p95_delta = $arangoRatioDelta
            comparable_changed = ([bool]$currentSummary.comparable -ne [bool]$baseline.comparable)
        }
    }
    Write-JsonLine -Path $devModeHistoryResolved -Value $currentSummary
    $report.dev_mode.summary_artifact = $devSummaryPath
    Write-TextNoBom -Path $devSummaryPath -Content ($currentSummary | ConvertTo-Json -Depth 50)
}

Write-TextNoBom -Path $reportPath -Content ($report | ConvertTo-Json -Depth 100)
Write-Host "Real-project workbench complete: $reportPath"
