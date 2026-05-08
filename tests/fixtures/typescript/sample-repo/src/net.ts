// Network-effect patterns.

import { raiseIfInvalid } from "./errors.js";

export async function fetchUser(id: number): Promise<unknown> {
  // expect: Net=definite, Async=definite, Throw=probable
  raiseIfInvalid(id);
  const res = await fetch(`https://api.example/users/${id}`);
  if (!res.ok) throw new Error(`status ${res.status}`);
  return res.json();
}

export async function postEvent(payload: unknown): Promise<void> {
  // expect: Net=definite, Async=definite
  await fetch("https://api.example/events", {
    method: "POST",
    body: JSON.stringify(payload),
  });
}

export function openWebSocket(url: string): WebSocket {
  // expect: Net=definite
  return new WebSocket(url);
}

export function makeRequestHelper(url: string): string {
  // expect: Net=possible
  // Pure body; name pattern only.
  return url.replace(/\/+$/, "");
}

declare interface RestClient {
  get(path: string): Promise<unknown>;
}

export async function fetchAllThings(client: RestClient): Promise<unknown[]> {
  // expect: Net=probable
  // `client` is annotated as a Client-typed value; methods are probable Net.
  return Promise.all([client.get("/a"), client.get("/b")]);
}
