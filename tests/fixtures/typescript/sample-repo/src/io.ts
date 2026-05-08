// IO-effect patterns.

import { writeFile, readFile } from "node:fs/promises";
import { writeFileSync } from "node:fs";

export async function writeLog(path: string, line: string): Promise<void> {
  // expect: IO=definite, Async=definite
  await writeFile(path, line + "\n", { flag: "a" });
}

export async function readConfig(path: string): Promise<string> {
  // expect: IO=definite, Async=definite
  return readFile(path, "utf-8");
}

export function emit(line: string): void {
  // expect: IO=definite
  console.log(line);
}

export function emitErr(line: string): void {
  // expect: IO=definite
  console.error(line);
}

export function dumpSync(path: string, data: string): void {
  // expect: IO=definite
  writeFileSync(path, data);
}

export function persistMaybe(path: string, _data: string): string {
  // expect: IO=possible
  // Pure body; name pattern only.
  return path;
}

export function setLocalstorage(key: string, value: string): void {
  // expect: IO=definite
  // Browser API.
  (globalThis as unknown as { localStorage: Storage }).localStorage.setItem(key, value);
}
