import { FormEvent, useState } from "react";
import { login, setToken } from "../api";

export function LoginPage() {
  const [username, setUsername] = useState("admin");
  const [password, setPassword] = useState("admin");
  const [error, setError] = useState<string | null>(null);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    setError(null);
    try {
      const resp = await login(username, password);
      setToken(resp.token);
      window.location.href = "/";
    } catch (err) {
      setError(err instanceof Error ? err.message : "Login failed");
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center p-6">
      <form
        onSubmit={onSubmit}
        className="w-full max-w-sm rounded border border-slate-800 bg-slate-900 p-6 space-y-4"
      >
        <h1 className="text-xl font-semibold">WireSentinel Cloud</h1>
        {error && <p className="text-red-400 text-sm">{error}</p>}
        <label className="block text-sm">
          Username
          <input
            className="mt-1 w-full rounded bg-slate-950 border border-slate-700 px-3 py-2"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
          />
        </label>
        <label className="block text-sm">
          Password
          <input
            type="password"
            className="mt-1 w-full rounded bg-slate-950 border border-slate-700 px-3 py-2"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
        </label>
        <button
          type="submit"
          className="w-full rounded bg-violet-700 hover:bg-violet-600 py-2 font-medium"
        >
          Continue
        </button>
      </form>
    </div>
  );
}
