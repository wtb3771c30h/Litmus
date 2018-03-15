/**
 * Minimal ambient declarations for the tiny slice of the Node.js runtime this
 * project uses. Declaring these locally keeps the toolchain fully
 * dependency-free: `npm install` pulls nothing, and `tsc` still type-checks.
 *
 * If you prefer the full, official typings, run `npm i -D @types/node` and add
