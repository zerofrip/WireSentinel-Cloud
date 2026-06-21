import { FormEvent, useEffect, useState } from "react";
import { createTeam, fetchTeams } from "../api";

export function TeamsPage() {
  const [teams, setTeams] = useState<Array<Record<string, unknown>>>([]);
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);

  function reload() {
    fetchTeams().then(setTeams).catch((e: Error) => setError(e.message));
  }

  useEffect(() => {
    reload();
  }, []);

  async function onSubmit(e: FormEvent) {
    e.preventDefault();
    try {
      await createTeam(name);
      setName("");
      reload();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed");
    }
  }

  if (error) return <p className="text-red-400">{error}</p>;

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">Teams</h1>
      <form onSubmit={onSubmit} className="flex gap-2 mb-6">
        <input
          className="rounded bg-slate-950 border border-slate-700 px-3 py-2"
          placeholder="Team name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <button type="submit" className="rounded bg-violet-700 px-4 py-2 text-sm">
          Create
        </button>
      </form>
      <ul className="space-y-2">
        {teams.map((t) => (
          <li key={String(t.id)} className="rounded border border-slate-800 p-3">
            {String(t.name)}
          </li>
        ))}
      </ul>
    </div>
  );
}
