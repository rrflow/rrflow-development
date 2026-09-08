/** Current plain session representation pending the H-04 credential boundary. */

import type { SuccessPayload } from "./operation.js";

export interface Session {
  principalId: string;
  lease: SuccessPayload<"session-create">;
}
