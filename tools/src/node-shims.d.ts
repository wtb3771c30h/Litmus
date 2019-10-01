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
