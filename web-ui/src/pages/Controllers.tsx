import { FormEvent, useEffect, useState } from "react";
import { fetchControllers, registerController } from "../api";

export function ControllersPage() {
  const [controllers, setControllers] = useState<Array<Record<string, unknown>>>([]);
  const [name, setName] = useState("");
  const [url, setUrl] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [error, setError] = useState<string | null>(null);

  function reload() {
    fetchControllers().then(setControllers).catch((e: Error) => setError(e.message));
  }

  useEffect(() => {
    reload();
  }, []);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    try {
      await registerController(name, url, apiKey);
      setName("");
      setUrl("");
      setApiKey("");
      reload();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed");
    }
  }

  if (error) return <p className="text-red-400">{error}</p>;

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">Federated Controllers</h1>
      <form onSubmit={onSubmit} className="grid gap-2 mb-6 max-w-md">
        <input
          className="rounded bg-slate-950 border border-slate-700 px-3 py-2"
          placeholder="Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <input
          className="rounded bg-slate-950 border border-slate-700 px-3 py-2"
          placeholder="Endpoint URL"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
        />
        <input
          className="rounded bg-slate-950 border border-slate-700 px-3 py-2"
          placeholder="API key"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
        />
        <button type="submit" className="rounded bg-violet-700 px-4 py-2 text-sm w-fit">
          Register
        </button>
      </form>
      <ul className="space-y-2">
        {controllers.map((c) => (
          <li key={String(c.id)} className="rounded border border-slate-800 p-3">
            <div className="font-medium">{String(c.name)}</div>
            <div className="text-xs text-slate-400">{String(c.endpoint_url)}</div>
            <div className="text-xs text-slate-500">Status: {String(c.status)}</div>
          </li>
        ))}
      </ul>
    </div>
  );
}
