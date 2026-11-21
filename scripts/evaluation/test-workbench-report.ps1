$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

# Exercise the real reporting statements without invoking the workload runner.
$sourcePath = Join-Path $PSScriptRoot "run-real-project-workbench.ps1"
$tokens = $null
$parseErrors = $null
$ast = [System.Management.Automation.Language.Parser]::ParseFile($sourcePath, [ref]$tokens, [ref]$parseErrors)
if ($parseErrors.Count -ne 0) { throw ($parseErrors | Out-String) }
$source = [System.IO.File]::ReadAllText($sourcePath)
$assignments = @($ast.FindAll({ param($node)
    $node -is [System.Management.Automation.Language.AssignmentStatementAst]
}, $true))
$start = @($assignments | Where-Object { $_.Left.Extent.Text -eq '$requireComparable' })
$end = @($assignments | Where-Object { $_.Left.Extent.Text -eq '$report.comparison' })
if ($start.Count -ne 1 -or $end.Count -ne 1) { throw "Reporting boundary is ambiguous." }
$comparisonBlock = [scriptblock]::Create($source.Substring(
    $start[0].Extent.StartOffset, $end[0].Extent.EndOffset - $start[0].Extent.StartOffset))
$tail = @($ast.EndBlock.Statements | Where-Object {
    $_.Extent.Text.StartsWith('Write-TextNoBom -Path $reportPath ')
})
if ($tail.Count -ne 1) { throw "Final report publication boundary is ambiguous." }
$publicationBlock = [scriptblock]::Create($source.Substring($tail[0].Extent.StartOffset))
foreach ($name in @('Has-Property', 'Write-TextNoBom')) {
    $definition = @($ast.FindAll({ param($node)
        $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq $name
    }, $true))
    if ($definition.Count -ne 1) { throw "Missing reporting dependency: $name" }
    . ([scriptblock]::Create($definition[0].Extent.Text))
}

$temporary = Join-Path ([System.IO.Path]::GetTempPath()) ("rrflow-report-" + [guid]::NewGuid())
[System.IO.Directory]::CreateDirectory($temporary) | Out-Null
$failures = [System.Collections.Generic.List[string]]::new()
try {
    foreach ($required in @($false, $true)) {
        foreach ($mask in 0..7) {
            $indexPassed = ($mask -band 1) -ne 0
            $retrievalPassed = ($mask -band 2) -ne 0
            $latencyPassed = ($mask -band 4) -ne 0
            $expected = $mask -eq 7
            $realTestPlan = @{ plan = [pscustomobject]@{
                acceptance = [pscustomobject]@{ require_comparable = $required }
                name = 'report-regression'
            }; sha256 = 'plan' }
            $chatExamples = @{ sha256 = 'prompts' }
            $surrealP95 = 2.0
            $arangoP95 = 3.0
            $ratioSurreal = 1.0
            $ratioArango = 1.0
            $comparatorResolved = 'reporting-fixture-only'
            $report = [ordered]@{ comparison = $null }
            $reportPath = Join-Path $temporary "$required-$mask.json"
            . $comparisonBlock
            $failed = $false
            try { . $publicationBlock 6>$null } catch {
                $failed = $true
                if (-not $_.Exception.Message.StartsWith('Required workbench checks failed. Results retained at ')) {
                    throw
                }
            }
            $saved = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
            if ($saved.comparison.comparable -ne $expected) {
                $failures.Add("required=$required outcomes=${mask}: report changed measured failure to success")
            }
            if ($failed -ne ($required -and -not $expected)) {
                $failures.Add("required=$required outcomes=${mask}: terminal failure did not match enforcement policy")
            }
        }
    }
} finally {
    # This unique directory contains only the sixteen reports created above.
    foreach ($file in [System.IO.Directory]::GetFiles($temporary)) { [System.IO.File]::Delete($file) }
    [System.IO.Directory]::Delete($temporary)
}
if ($failures.Count -gt 0) { throw ($failures -join "`n") }
Write-Host "workbench-report: PASS: 16 outcome/policy cases, persisted reports and terminal failure"
