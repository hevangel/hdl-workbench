import type { z } from "zod";
import { ZodError } from "zod";
import { RpcErrorSchema } from "@xezim-ide/protocol";

export type BridgeErrorCode = z.infer<typeof RpcErrorSchema>["code"];

export class BridgeError extends Error {
  constructor(
    readonly code: BridgeErrorCode,
    message: string,
    readonly retryable?: boolean,
  ) {
    super(message);
    this.name = "BridgeError";
  }

  toJSON(): { code: BridgeErrorCode; message: string; retryable?: boolean } {
    return {
      code: this.code,
      message: this.message,
      ...(this.retryable === undefined ? {} : { retryable: this.retryable }),
    };
  }
}

export function asBridgeError(error: unknown): BridgeError {
  if (error instanceof BridgeError) return error;
  if (error instanceof ZodError) return new BridgeError("INVALID_REQUEST", "Request did not match the HDL Workbench protocol schema");
  if (error instanceof Error) return new BridgeError("INTERNAL_ERROR", error.message);
  return new BridgeError("INTERNAL_ERROR", "Unexpected bridge error");
}
