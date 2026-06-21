import { useEffect, useState } from "react";
import { fetchMe } from "../api";

export function UsersPage() {
  const [me, setMe] = useState<Record<string, string> | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchMe()
      .then(setMe)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <p className="text-red-400">{error}</p>;
  if (!me) return <p>Loading…</p>;

  return (
    <div>
      <h1 className="text-2xl font-semibold mb-6">Users</h1>
      <p className="text-sm text-slate-400 mb-4">
        Phase 11 exposes the authenticated user profile. Full user management is planned for a later phase.
      </p>
      <div className="rounded border border-slate-800 p-4 max-w-md">
        <dl className="space-y-2 text-sm">
          <div className="flex justify-between">
            <dt className="text-slate-400">Username</dt>
            <dd>{me.username}</dd>
          </div>
          <div className="flex justify-between">
            <dt className="text-slate-400">Role</dt>
            <dd>{me.role}</dd>
          </div>
          <div className="flex justify-between">
            <dt className="text-slate-400">Tenant</dt>
            <dd className="truncate max-w-[12rem]">{me.tenant_id}</dd>
          </div>
        </dl>
      </div>
    </div>
  );
}
