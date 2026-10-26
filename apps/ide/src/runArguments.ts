/** Longest command line the Arguments field accepts, in characters. */
export const MAX_RUN_ARGUMENTS_LENGTH = 4096;

export class RunArgumentsError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "RunArgumentsError";
  }
}

/**
 * Turn a simulator command-line fragment into plusargs. The browser build of Xezim
 * has no process arguments, so it takes the `+name[=value]` arguments that
 * `$test$plusargs` and `$value$plusargs` read, plus `-sv_seed <n|random>` as in
 * the native CLI. Values cannot contain spaces. Anything else would be silently ignored by the simulator, so it
 * is rejected here instead.
 */
export function parseRunArguments(line: string): string[] {
  if (line.length > MAX_RUN_ARGUMENTS_LENGTH) throw new RunArgumentsError(`Simulator arguments are limited to ${MAX_RUN_ARGUMENTS_LENGTH} characters.`);
  const tokens = line.split(/\s+/).filter(Boolean);
  const plusargs: string[] = [];
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index] as string;
    if (token === "-sv_seed") {
      const seed = tokens[index + 1];
      if (seed === undefined || !(seed.toLowerCase() === "random" || /^\d+$/.test(seed))) {
        throw new RunArgumentsError("-sv_seed needs a non-negative integer or 'random'.");
      }
      plusargs.push(`+seed=${seed}`);
      index += 1;
    } else if (/^\+[^\s]+$/.test(token)) {
      plusargs.push(token);
    } else {
      throw new RunArgumentsError(`Unsupported argument '${token}'. The browser simulator accepts +name[=value] plusargs and -sv_seed <n>.`);
    }
  }
  if (plusargs.length > 64) throw new RunArgumentsError("At most 64 plusargs are supported.");
  return plusargs;
}
