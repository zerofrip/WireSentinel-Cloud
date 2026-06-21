import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { Layout } from "./components/Layout";
import { getToken } from "./api";
import { AnonymityFleetPage } from "./pages/AnonymityFleet";
import { AnonymityPrivacyAnalyticsPage } from "./pages/AnonymityPrivacyAnalytics";
import { BillingPage } from "./pages/Billing";
import { CompliancePage } from "./pages/Compliance";
import { ControllersPage } from "./pages/Controllers";
import { DashboardPage } from "./pages/Dashboard";
import { FederationMonitoringPage } from "./pages/FederationMonitoring";
import { FederationPage } from "./pages/Federation";
import { KernelFleetOverviewPage } from "./pages/KernelFleetOverview";
import { KernelMonitoringPage } from "./pages/KernelMonitoring";
import { LoginPage } from "./pages/Login";
import { ObservabilityPage } from "./pages/Observability";
import { OrganizationsPage } from "./pages/Organizations";
import { QuotasPage } from "./pages/Quotas";
import { RecoveryPage } from "./pages/Recovery";
import { RegionsPage } from "./pages/Regions";
import { SubscriptionsPage } from "./pages/Subscriptions";
import { TeamsPage } from "./pages/Teams";
import { TenantsPage } from "./pages/Tenants";
import { UsagePage } from "./pages/Usage";
import { UsersPage } from "./pages/Users";
import { IdentityPage } from "./pages/Identity";
import { AccessPoliciesPage } from "./pages/AccessPolicies";
import { TrustAnalyticsPage } from "./pages/TrustAnalytics";
import { ResourcePublishingPage } from "./pages/ResourcePublishing";
import { ThreatCenterPage } from "./pages/ThreatCenter";
import { CasbAnalyticsPage } from "./pages/CasbAnalytics";
import { DlpAnalyticsPage } from "./pages/DlpAnalytics";
import { RiskAnalyticsPage } from "./pages/RiskAnalytics";
import { UebaAnalyticsPage } from "./pages/UebaAnalytics";
import { XdrThreatCenterPage } from "./pages/XdrThreatCenter";
import { XdrIncidentCenterPage } from "./pages/XdrIncidentCenter";
import { XdrDetectionCoveragePage } from "./pages/XdrDetectionCoverage";
import { XdrMitreCoveragePage } from "./pages/XdrMitreCoverage";
import { XdrFleetAnalyticsPage } from "./pages/XdrFleetAnalytics";
import { CnappDashboardPage } from "./pages/CnappDashboard";
import { CnappRiskCenterPage } from "./pages/CnappRiskCenter";
import { CnappComplianceCenterPage } from "./pages/CnappComplianceCenter";
import { CnappVulnerabilityCenterPage } from "./pages/CnappVulnerabilityCenter";
import { CnappAttackPathsPage } from "./pages/CnappAttackPaths";
import { CnappMultiCloudAnalyticsPage } from "./pages/CnappMultiCloudAnalytics";
import { AiDashboardPage } from "./pages/AiDashboard";
import { AiInvestigationsPage } from "./pages/AiInvestigations";
import { AiRiskCenterPage } from "./pages/AiRiskCenter";
import { AiReportsPage } from "./pages/AiReports";
import { AiCompliancePage } from "./pages/AiCompliance";
import { AiForecastingPage } from "./pages/AiForecasting";
import { WiresockSplitTemplatesPage } from "./pages/WiresockSplitTemplates";
import { WiresockTcpTerminationPage } from "./pages/WiresockTcpTermination";
import { WiresockHandshakeProxyPage } from "./pages/WiresockHandshakeProxy";

function RequireAuth({ children }: { children: React.ReactNode }) {
  if (!getToken()) return <Navigate to="/login" replace />;
  return <>{children}</>;
}

export function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route
          element={
            <RequireAuth>
              <Layout />
            </RequireAuth>
          }
        >
          <Route index element={<DashboardPage />} />
          <Route path="tenants" element={<TenantsPage />} />
          <Route path="organizations" element={<OrganizationsPage />} />
          <Route path="teams" element={<TeamsPage />} />
          <Route path="controllers" element={<ControllersPage />} />
          <Route path="compliance" element={<CompliancePage />} />
          <Route path="federation" element={<FederationPage />} />
          <Route path="subscriptions" element={<SubscriptionsPage />} />
          <Route path="billing" element={<BillingPage />} />
          <Route path="usage" element={<UsagePage />} />
          <Route path="quotas" element={<QuotasPage />} />
          <Route path="regions" element={<RegionsPage />} />
          <Route path="recovery" element={<RecoveryPage />} />
          <Route path="observability" element={<ObservabilityPage />} />
          <Route path="users" element={<UsersPage />} />
          <Route path="kernel" element={<KernelMonitoringPage />} />
          <Route path="kernel/fleet" element={<KernelFleetOverviewPage />} />
          <Route path="anonymity" element={<AnonymityFleetPage />} />
          <Route path="anonymity/analytics" element={<AnonymityPrivacyAnalyticsPage />} />
          <Route path="anonymity/federation" element={<FederationMonitoringPage />} />
          <Route path="identity" element={<IdentityPage />} />
          <Route path="ztna/policies" element={<AccessPoliciesPage />} />
          <Route path="ztna/trust" element={<TrustAnalyticsPage />} />
          <Route path="ztna/resources" element={<ResourcePublishingPage />} />
          <Route path="sse/threats" element={<ThreatCenterPage />} />
          <Route path="sse/casb" element={<CasbAnalyticsPage />} />
          <Route path="sse/dlp" element={<DlpAnalyticsPage />} />
          <Route path="sse/risk" element={<RiskAnalyticsPage />} />
          <Route path="sse/ueba" element={<UebaAnalyticsPage />} />
          <Route path="xdr/threats" element={<XdrThreatCenterPage />} />
          <Route path="xdr/incidents" element={<XdrIncidentCenterPage />} />
          <Route path="xdr/detections" element={<XdrDetectionCoveragePage />} />
          <Route path="xdr/mitre" element={<XdrMitreCoveragePage />} />
          <Route path="xdr/analytics" element={<XdrFleetAnalyticsPage />} />
          <Route path="cnapp" element={<CnappDashboardPage />} />
          <Route path="cnapp/risk" element={<CnappRiskCenterPage />} />
          <Route path="cnapp/compliance" element={<CnappComplianceCenterPage />} />
          <Route path="cnapp/vulnerabilities" element={<CnappVulnerabilityCenterPage />} />
          <Route path="cnapp/attack-paths" element={<CnappAttackPathsPage />} />
          <Route path="cnapp/analytics" element={<CnappMultiCloudAnalyticsPage />} />
          <Route path="ai" element={<AiDashboardPage />} />
          <Route path="ai/investigations" element={<AiInvestigationsPage />} />
          <Route path="ai/risk" element={<AiRiskCenterPage />} />
          <Route path="ai/reports" element={<AiReportsPage />} />
          <Route path="ai/compliance" element={<AiCompliancePage />} />
          <Route path="ai/forecasting" element={<AiForecastingPage />} />
          <Route path="wiresock/split-templates" element={<WiresockSplitTemplatesPage />} />
          <Route path="wiresock/tcp-termination" element={<WiresockTcpTerminationPage />} />
          <Route path="wiresock/handshake-proxy" element={<WiresockHandshakeProxyPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
