const TOKEN_KEY = "ws_cloud_token";
const TENANT_KEY = "ws_cloud_tenant";

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string) {
  localStorage.setItem(TOKEN_KEY, token);
}

export function clearToken() {
  localStorage.removeItem(TOKEN_KEY);
}

export function getTenantId(): string | null {
  return localStorage.getItem(TENANT_KEY);
}

export function setTenantId(tenantId: string) {
  localStorage.setItem(TENANT_KEY, tenantId);
}

export function clearTenantId() {
  localStorage.removeItem(TENANT_KEY);
}

async function apiFetch<T>(path: string, init: RequestInit = {}): Promise<T> {
  const headers = new Headers(init.headers);
  headers.set("Content-Type", "application/json");
  const token = getToken();
  if (token) headers.set("Authorization", `Bearer ${token}`);
  const tenantId = getTenantId();
  if (tenantId) headers.set("X-Tenant-Id", tenantId);

  const resp = await fetch(path, { ...init, headers });
  if (!resp.ok) {
    const body = await resp.text();
    throw new Error(body || resp.statusText);
  }
  if (resp.status === 204) return undefined as T;
  return resp.json() as Promise<T>;
}

export async function login(username: string, password: string, tenantId?: string) {
  const resp = await apiFetch<{
    token: string;
    username: string;
    role: string;
    tenant_id: string;
  }>("/api/v1/auth/login", {
    method: "POST",
    body: JSON.stringify({ username, password, tenant_id: tenantId }),
  });
  setTenantId(resp.tenant_id);
  return resp;
}

export async function fetchMe() {
  return apiFetch<{ user_id: string; username: string; role: string; tenant_id: string }>(
    "/api/v1/auth/me",
  );
}

export async function fetchTenants() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/tenants");
}

export async function fetchOrganizations() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/organizations");
}

export async function fetchTeams() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/teams");
}

export async function fetchControllers() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/federation/controllers");
}

export async function fetchSync() {
  return apiFetch<{ entities: unknown[]; conflicts: unknown[] }>("/api/v1/cloud/sync");
}

export async function fetchCompliance() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/compliance");
}

export async function fetchMetrics() {
  return apiFetch<Record<string, number>>("/api/v1/cloud/metrics");
}

export async function fetchSubscriptions() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/subscriptions");
}

export async function fetchPlans() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/plans");
}

export async function fetchHealth() {
  return apiFetch<{ status: string; service: string }>("/health");
}

export async function createOrganization(name: string) {
  return apiFetch<Record<string, unknown>>("/api/v1/organizations", {
    method: "POST",
    body: JSON.stringify({ name }),
  });
}

export async function createTeam(name: string) {
  return apiFetch<Record<string, unknown>>("/api/v1/teams", {
    method: "POST",
    body: JSON.stringify({ name }),
  });
}

export async function runCompliance() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/compliance", {
    method: "POST",
  });
}

export async function registerController(name: string, endpointUrl: string, apiKey: string) {
  return apiFetch<Record<string, unknown>>("/api/v1/federation/controllers", {
    method: "POST",
    body: JSON.stringify({ name, endpoint_url: endpointUrl, api_key: apiKey }),
  });
}

export interface KernelFleetRollup {
  id: string;
  tenant_id: string;
  controller_id?: string | null;
  reporting_devices: number;
  healthy_devices: number;
  kernel_devices: number;
  ndis_devices: number;
  total_active_routes: number;
  rolled_up_at: string;
}

export interface KernelFleetOverview {
  tenant_id: string;
  reporting_devices: number;
  healthy_devices: number;
  kernel_devices: number;
  ndis_devices: number;
  stub_devices: number;
  total_active_routes: number;
  controllers_reporting: number;
  rollups: KernelFleetRollup[];
}

export interface KernelFleetStatistics {
  tenant_id: string;
  classify_count: number;
  packets_per_sec: number;
  avg_healthy_ratio: number;
  kernel_adoption_ratio: number;
  rollups_recorded: number;
}

export async function fetchCloudKernel() {
  return apiFetch<KernelFleetOverview>("/api/v1/cloud/kernel");
}

export async function fetchCloudKernelStatistics() {
  return apiFetch<KernelFleetStatistics>("/api/v1/cloud/kernel/statistics");
}

export interface AnonymityFleetRollup {
  id: string;
  tenant_id: string;
  controller_id?: string | null;
  reporting_devices: number;
  healthy_devices: number;
  connected_devices: number;
  federation_peers_total: number;
  avg_anonymity_score: number;
  avg_entropy_bits: number;
  total_active_routes: number;
  rolled_up_at: string;
}

export interface AnonymityFleetOverview {
  tenant_id: string;
  reporting_devices: number;
  healthy_devices: number;
  connected_devices: number;
  federation_peers_total: number;
  avg_anonymity_score: number;
  total_active_routes: number;
  controllers_reporting: number;
  rollups: AnonymityFleetRollup[];
}

export interface AnonymityPrivacyAnalytics {
  tenant_id: string;
  avg_entropy_bits: number;
  avg_route_entropy: number;
  avg_anonymity_score: number;
  federation_peers_total: number;
  healthy_ratio: number;
  rollups_recorded: number;
}

export async function fetchCloudAnonymity() {
  return apiFetch<AnonymityFleetOverview>("/api/v1/cloud/anonymity");
}

export async function fetchCloudAnonymityAnalytics() {
  return apiFetch<AnonymityPrivacyAnalytics>("/api/v1/cloud/anonymity/analytics");
}

export async function fetchBillingPlans() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/billing/plans");
}

export async function fetchBillingSubscription() {
  return apiFetch<Record<string, unknown> | null>("/api/v1/billing/subscription");
}

export async function fetchBillingInvoices() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/billing/invoices");
}

export async function fetchQuotas() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/quotas");
}

export async function fetchRegions() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/regions");
}

export async function fetchRegionHealth() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/regions/health");
}

export async function fetchRecoveryRuns() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/recovery/runs");
}

export async function fetchUsageAggregates() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/cloud/usage");
}

export async function fetchObservabilityMetrics() {
  return apiFetch<Record<string, unknown>>("/api/v1/observability/metrics");
}

export async function fetchLogs(limit = 100) {
  return apiFetch<Array<Record<string, unknown>>>(`/api/v1/logs?limit=${limit}`);
}

export interface ZtnaFleetOverview {
  tenant_id: string;
  reporting_devices: number;
  avg_trust_score: number;
  allow_count: number;
  deny_count: number;
  challenge_count: number;
  published_resources: number;
}

export interface ZtnaAnalyticsSummary {
  tenant_id: string;
  avg_trust_score: number;
  allow_count: number;
  deny_count: number;
  challenge_count: number;
  published_resources: number;
  deny_ratio: number;
  rollups_recorded: number;
}

export async function fetchCloudZtna() {
  return apiFetch<ZtnaFleetOverview>("/api/v1/cloud/ztna");
}

export async function fetchCloudZtnaAnalytics() {
  return apiFetch<ZtnaAnalyticsSummary>("/api/v1/cloud/ztna/analytics");
}

export interface SseFleetOverview {
  tenant_id: string;
  reporting_devices: number;
  swg_requests: number;
  swg_blocked: number;
  threat_count: number;
  casb_incidents: number;
  dlp_incidents: number;
  avg_risk_score: number;
  ueba_alerts: number;
  controllers_reporting: number;
}

export interface SseAnalyticsSummary {
  tenant_id: string;
  block_ratio: number;
  avg_risk_score: number;
  threat_count: number;
  casb_incidents: number;
  dlp_incidents: number;
  ueba_alerts: number;
  rollups_recorded: number;
}

export async function fetchCloudSse() {
  return apiFetch<SseFleetOverview>("/api/v1/cloud/sse");
}

export async function fetchCloudSseAnalytics() {
  return apiFetch<SseAnalyticsSummary>("/api/v1/cloud/sse/analytics");
}

export async function fetchSsePolicies() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/sse/policies");
}

export interface XdrFleetOverview {
  tenant_id: string;
  reporting_devices: number;
  total_incidents: number;
  open_incidents: number;
  critical_incidents: number;
  total_detections: number;
  active_hunts: number;
  mitre_techniques_detected: number;
  mitre_coverage_pct: number;
  avg_incident_mttr_hours: number;
  fleet_threat_score: number;
  controllers_reporting: number;
}

export interface XdrAnalyticsSummary {
  tenant_id: string;
  total_incidents: number;
  open_incidents: number;
  critical_incidents: number;
  total_detections: number;
  mitre_techniques_detected: number;
  mitre_coverage_pct: number;
  avg_incident_mttr_hours: number;
  fleet_threat_score: number;
  rollups_recorded: number;
}

export interface XdrIncidentRecord {
  id: string;
  tenant_id: string;
  controller_id?: string;
  title: string;
  status: string;
  severity: string;
  detection_count: number;
  opened_at: string;
  resolved_at?: string;
}

export interface XdrDetectionRecord {
  id: string;
  tenant_id: string;
  controller_id?: string;
  rule_name: string;
  rule_kind: string;
  severity: string;
  mitre_technique_id?: string;
  device_id?: string;
  matched_at: string;
}

export interface XdrMitreCoverageRecord {
  id: string;
  tenant_id: string;
  tactic: string;
  technique_id: string;
  technique_name: string;
  detection_count: number;
  coverage_pct: number;
  last_seen_at?: string;
}

export async function fetchCloudXdr() {
  return apiFetch<XdrFleetOverview>("/api/v1/cloud/xdr");
}

export async function fetchCloudXdrAnalytics() {
  return apiFetch<XdrAnalyticsSummary>("/api/v1/cloud/xdr/analytics");
}

export async function fetchCloudXdrIncidents() {
  return apiFetch<XdrIncidentRecord[]>("/api/v1/cloud/xdr/incidents");
}

export async function fetchCloudXdrDetections() {
  return apiFetch<XdrDetectionRecord[]>("/api/v1/cloud/xdr/detections");
}

export async function fetchCloudXdrMitreCoverage() {
  return apiFetch<XdrMitreCoverageRecord[]>("/api/v1/cloud/xdr/mitre-coverage");
}

export interface CnappFleetOverview {
  tenant_id: string;
  reporting_accounts: number;
  posture_score: number;
  compliance_pct: number;
  open_vulnerabilities: number;
  critical_vulnerabilities: number;
  attack_paths_detected: number;
  multi_cloud_providers: number;
  fleet_risk_score: number;
  controllers_reporting: number;
  attack_paths: CnappAttackPathRecord[];
}

export interface CnappAnalyticsSummary {
  tenant_id: string;
  posture_score: number;
  compliance_pct: number;
  open_vulnerabilities: number;
  critical_vulnerabilities: number;
  attack_paths_detected: number;
  multi_cloud_providers: number;
  fleet_risk_score: number;
  rollups_recorded: number;
}

export interface CnappPostureRecord {
  id: string;
  tenant_id: string;
  controller_id?: string;
  cloud_provider: string;
  account_id?: string;
  resource_kind: string;
  posture_score: number;
  risk_level: string;
  findings_count: number;
  assessed_at: string;
}

export interface CnappComplianceRecord {
  id: string;
  tenant_id: string;
  controller_id?: string;
  framework: string;
  control_id: string;
  control_name: string;
  status: string;
  compliance_pct: number;
  last_checked_at?: string;
}

export interface CnappVulnerabilityRecord {
  id: string;
  tenant_id: string;
  controller_id?: string;
  cve_id?: string;
  title: string;
  severity: string;
  resource_id?: string;
  cloud_provider?: string;
  status: string;
  discovered_at: string;
}

export interface CnappAttackPathRecord {
  id: string;
  tenant_id: string;
  controller_id?: string;
  name: string;
  severity: string;
  path_length: number;
  entry_point?: string;
  target_asset?: string;
  status: string;
  discovered_at: string;
}

export async function fetchCloudCnapp() {
  return apiFetch<CnappFleetOverview>("/api/v1/cloud/cnapp");
}

export async function fetchCloudCnappAnalytics() {
  return apiFetch<CnappAnalyticsSummary>("/api/v1/cloud/cnapp/analytics");
}

export async function fetchCloudCnappPosture() {
  return apiFetch<CnappPostureRecord[]>("/api/v1/cloud/cnapp/posture");
}

export async function fetchCloudCnappCompliance() {
  return apiFetch<CnappComplianceRecord[]>("/api/v1/cloud/cnapp/compliance");
}

export async function fetchCloudCnappVulnerabilities() {
  return apiFetch<CnappVulnerabilityRecord[]>("/api/v1/cloud/cnapp/vulnerabilities");
}

export interface AiFleetOverview {
  tenant_id: string;
  reporting_agents: number;
  open_investigations: number;
  critical_risks: number;
  total_correlations: number;
  compliance_pct: number;
  avg_risk_score: number;
  prompt_injection_events: number;
  data_exfiltration_events: number;
  fleet_ai_risk_score: number;
  controllers_reporting: number;
  investigations: AiInvestigationRecord[];
}

export interface AiAnalyticsSummary {
  tenant_id: string;
  reporting_agents: number;
  open_investigations: number;
  critical_risks: number;
  total_correlations: number;
  compliance_pct: number;
  avg_risk_score: number;
  prompt_injection_events: number;
  data_exfiltration_events: number;
  fleet_ai_risk_score: number;
  rollups_recorded: number;
}

export interface AiInvestigationRecord {
  id: string;
  tenant_id: string;
  controller_id?: string;
  title: string;
  status: string;
  severity: string;
  category: string;
  model_name?: string;
  agent_id?: string;
  finding_count: number;
  opened_at: string;
}

export interface AiRiskRecord {
  id: string;
  tenant_id: string;
  controller_id?: string;
  risk_category: string;
  risk_score: number;
  severity: string;
  model_name?: string;
  resource_id?: string;
  status: string;
  assessed_at: string;
}

export interface AiReportRecord {
  id: string;
  tenant_id: string;
  controller_id?: string;
  report_type: string;
  title: string;
  status: string;
  compliance_pct: number;
  period_start?: string;
  period_end?: string;
  generated_at: string;
}

export async function fetchCloudAi() {
  return apiFetch<AiFleetOverview>("/api/v1/cloud/ai");
}

export async function fetchCloudAiAnalytics() {
  return apiFetch<AiAnalyticsSummary>("/api/v1/cloud/ai/analytics");
}

export async function fetchCloudAiRisk() {
  return apiFetch<AiRiskRecord[]>("/api/v1/cloud/ai/risk");
}

export async function fetchCloudAiReports() {
  return apiFetch<AiReportRecord[]>("/api/v1/cloud/ai/reports");
}

export async function fetchCloudAiInvestigations() {
  return apiFetch<AiInvestigationRecord[]>("/api/v1/cloud/ai/investigations");
}

export interface VpnGatewayCompatFleetOverview {
  tenant_id: string;
  reporting_endpoints: number;
  active_split_templates: number;
  tcp_termination_rules: number;
  handshake_proxy_active: number;
  bypass_events: number;
  fleet_health_score: number;
  controllers_reporting: number;
  split_templates: VpnGatewayCompatSplitTemplateRecord[];
}

export interface VpnGatewayCompatSplitTemplateRecord {
  id: string;
  tenant_id: string;
  controller_id?: string;
  name: string;
  description: string;
  template_mode: string;
  enabled: boolean;
  app_rules_count: number;
  domain_rules_count: number;
  synced_at: string;
}

export interface VpnGatewayCompatTcpTerminationRecord {
  id: string;
  tenant_id: string;
  controller_id?: string;
  mode: string;
  rule_name: string;
  process_name?: string;
  profile_id?: string;
  enabled: boolean;
  synced_at: string;
}

export interface VpnGatewayCompatHandshakeProxyRecord {
  id: string;
  tenant_id: string;
  controller_id?: string;
  name: string;
  proxy_type: string;
  endpoint?: string;
  enabled: boolean;
  synced_at: string;
}

export async function fetchCloudSplitTemplates() {
  return apiFetch<VpnGatewayCompatFleetOverview>("/api/v1/cloud/split-templates");
}

export async function fetchCloudTcpTermination() {
  return apiFetch<VpnGatewayCompatTcpTerminationRecord[]>("/api/v1/cloud/tcp-termination");
}

export async function fetchCloudHandshakeProxy() {
  return apiFetch<VpnGatewayCompatHandshakeProxyRecord[]>("/api/v1/cloud/handshake-proxy");
}

export async function fetchIdentityProviders() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/identity/providers");
}

export async function fetchPublishedResources() {
  return apiFetch<Array<Record<string, unknown>>>("/api/v1/resources");
}
