/**
 * Minimal ambient declarations for the tiny slice of the Node.js runtime this
 * project uses. Declaring these locally keeps the toolchain fully
 * dependency-free: `npm install` pulls nothing, and `tsc` still type-checks.
 *
 * If you prefer the full, official typings, run `npm i -D @types/node` and add
 * `"types": ["node"]` to tsconfig.json — this file then becomes redundant.
 */

declare module "node:fs" {
  /** Read a file (or fd 0 for stdin) as text. */
  export function readFileSync(path: string | number, encoding: "utf8"): string;
  export function writeFileSync(path: string, data: string): void;
}

declare module "node:test" {
  type TestFn = () => void | Promise<void>;
  export function test(name: string, fn: TestFn): void;
}

declare module "node:assert/strict" {
  interface Assert {
    (value: unknown, message?: string): void;
    equal(actual: unknown, expected: unknown, message?: string): void;
    deepEqual(actual: unknown, expected: unknown, message?: string): void;
    match(value: string, regexp: RegExp, message?: string): void;
    doesNotMatch(value: string, regexp: RegExp, message?: string): void;
    throws(fn: () => unknown, expected?: RegExp | (new (...args: never[]) => Error)): void;
  }
  const assert: Assert;
  export default assert;
}

interface ProcessStream {
  write(data: string): void;
}

interface Process {
  argv: string[];
  exitCode?: number;
  stdout: ProcessStream;
  stderr: ProcessStream;
}

declare const process: Process;
