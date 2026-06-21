import { NavLink, Outlet } from "react-router-dom";
import { clearTenantId, clearToken } from "../api";

const links = [
  { to: "/", label: "Dashboard" },
  { to: "/billing", label: "Billing" },
  { to: "/usage", label: "Usage" },
  { to: "/quotas", label: "Quotas" },
  { to: "/regions", label: "Regions" },
  { to: "/recovery", label: "Recovery" },
  { to: "/observability", label: "Observability" },
  { to: "/tenants", label: "Tenants" },
  { to: "/organizations", label: "Organizations" },
  { to: "/teams", label: "Teams" },
  { to: "/controllers", label: "Controllers" },
  { to: "/federation", label: "Federation" },
  { to: "/compliance", label: "Compliance" },
  { to: "/kernel", label: "Kernel Monitoring" },
  { to: "/kernel/fleet", label: "Kernel Fleet" },
  { to: "/anonymity", label: "Anonymity Fleet" },
  { to: "/anonymity/analytics", label: "Anonymity Analytics" },
  { to: "/anonymity/federation", label: "Federation Monitoring" },
  { to: "/identity", label: "Identity" },
  { to: "/ztna/policies", label: "Access Policies" },
  { to: "/ztna/trust", label: "Trust Analytics" },
  { to: "/ztna/resources", label: "Resource Publishing" },
  { to: "/sse/threats", label: "Threat Center" },
  { to: "/sse/casb", label: "CASB Analytics" },
  { to: "/sse/dlp", label: "DLP Analytics" },
  { to: "/sse/risk", label: "Risk Analytics" },
  { to: "/sse/ueba", label: "UEBA Analytics" },
  { to: "/xdr/threats", label: "XDR Threat Center" },
  { to: "/xdr/incidents", label: "XDR Incidents" },
  { to: "/xdr/detections", label: "XDR Detections" },
  { to: "/xdr/mitre", label: "XDR MITRE Coverage" },
  { to: "/xdr/analytics", label: "XDR Fleet Analytics" },
  { to: "/cnapp", label: "CNAPP Dashboard" },
  { to: "/cnapp/risk", label: "CNAPP Risk Center" },
  { to: "/cnapp/compliance", label: "CNAPP Compliance" },
  { to: "/cnapp/vulnerabilities", label: "CNAPP Vulnerabilities" },
  { to: "/cnapp/attack-paths", label: "CNAPP Attack Paths" },
  { to: "/cnapp/analytics", label: "CNAPP Multi-Cloud Analytics" },
  { to: "/ai", label: "AI Security Dashboard" },
  { to: "/ai/investigations", label: "AI Investigations" },
  { to: "/ai/risk", label: "AI Risk Center" },
  { to: "/ai/reports", label: "AI Security Reports" },
  { to: "/ai/compliance", label: "AI Compliance" },
  { to: "/ai/forecasting", label: "AI Risk Forecasting" },
  { to: "/subscriptions", label: "Subscriptions" },
  { to: "/users", label: "Users" },
];

export function Layout() {
  return (
    <div className="min-h-screen flex">
      <aside className="w-56 bg-slate-900 border-r border-slate-800 p-4 flex flex-col gap-2 overflow-y-auto max-h-screen">
        <div className="text-lg font-semibold mb-4">WireSentinel Cloud</div>
        {links.map((l) => (
          <NavLink
            key={l.to}
            to={l.to}
            end={l.to === "/"}
            className={({ isActive }) =>
              `rounded px-3 py-2 text-sm ${isActive ? "bg-violet-900 text-violet-100" : "hover:bg-slate-800"}`
            }
          >
            {l.label}
          </NavLink>
        ))}
        <button
          className="mt-auto text-left text-sm text-slate-400 hover:text-white"
          onClick={() => {
            clearToken();
            clearTenantId();
            window.location.href = "/login";
          }}
        >
          Sign out
        </button>
      </aside>
      <main className="flex-1 p-8">
        <Outlet />
      </main>
    </div>
  );
}
