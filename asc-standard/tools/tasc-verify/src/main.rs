use anyhow::{anyhow, bail, Context, Result};
use base64::Engine;
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

#[derive(Debug, Parser)]
#[command(name = "tasc-verify")]
#[command(about = "Verify TASC Assurance Pack artifacts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Verify(Box<VerifyArgs>),
    Report(ReportArgs),
    CheckProof(CheckProofArgs),
}

#[derive(Debug, Args)]
struct VerifyArgs {
    #[arg(long)]
    bundle: PathBuf,
    #[arg(long)]
    profile: String,
    #[arg(long, default_value = "eu-north-star")]
    policy: String,
    #[arg(long, default_value = "TA2")]
    require_ta: String,
    #[arg(long, default_value = "rekor,mirror")]
    require_transparency: String,
    #[arg(long, default_value = "spec/tasc/checks.yaml")]
    checks_file: PathBuf,
    #[arg(
        long,
        default_value = "policies/transparency/trusted-log-checkpoints.json"
    )]
    trusted_checkpoints: PathBuf,
    #[arg(long, default_value = "policies/badge-registry.json")]
    badge_registry: PathBuf,
    #[arg(long, default_value = "policies/attestation/trust-policy.json")]
    attestation_trust_policy: PathBuf,
    #[arg(long, default_value = "policies/attestation/pki/trust-roots.pem")]
    trust_roots: PathBuf,
    #[arg(long, default_value = "policies/attestation/revocation-snapshot.json")]
    revocation_snapshot: PathBuf,
    #[arg(long, default_value = "policies/provenance/freshness-policy.yaml")]
    freshness_policy: PathBuf,
    #[arg(
        long,
        default_value = "policies/transparency/verification-policy.yaml"
    )]
    transparency_policy: PathBuf,
    #[arg(long, default_value = "spec/tasc/remediation.yaml")]
    remediation_file: PathBuf,
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct ReportArgs {
    #[arg(long)]
    input: PathBuf,
}

#[derive(Debug, Args)]
struct CheckProofArgs {
    #[arg(long)]
    proof: PathBuf,
    #[arg(
        long,
        default_value = "policies/transparency/trusted-log-checkpoints.json"
    )]
    trusted_checkpoints: PathBuf,
}

#[derive(Debug, Deserialize)]
struct CheckCatalog {
    checks: Vec<CheckSpec>,
}

#[derive(Debug, Deserialize)]
struct CheckSpec {
    id: String,
}

#[derive(Debug, Deserialize)]
struct RemediationCatalog {
    checks: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CheckResult {
    id: String,
    result: String,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerifyReport {
    #[serde(rename = "tascVerifyVersion")]
    tasc_verify_version: String,
    #[serde(rename = "specVersion")]
    spec_version: String,
    profile: String,
    policy: String,
    #[serde(rename = "bundleDigest")]
    bundle_digest: String,
    #[serde(rename = "requireTa")]
    require_ta: String,
    #[serde(rename = "requireTransparency")]
    require_transparency: Vec<String>,
    result: String,
    checks: Vec<CheckResult>,
    #[serde(rename = "failedChecks")]
    failed_checks: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TopologyNode {
    id: String,
    #[serde(rename = "type")]
    node_type: String,
}

#[derive(Debug, Deserialize)]
struct TopologyEdge {
    from: String,
    to: String,
    signal: String,
}

#[derive(Debug, Deserialize)]
struct TopologyAssertion {
    id: String,
    status: String,
    #[serde(rename = "proofRef")]
    proof_ref: String,
}

#[derive(Debug, Deserialize)]
struct ArtifactRef {
    path: String,
    sha256: String,
}

#[derive(Debug, Deserialize)]
struct SignedLog {
    #[serde(rename = "schemaVersion")]
    schema_version: String,
    #[serde(rename = "signerKeyId")]
    signer_key_id: String,
    signature: String,
    #[serde(default, rename = "signatureAlgorithm")]
    signature_algorithm: String,
    #[serde(default, rename = "signatureEncoding")]
    signature_encoding: String,
    #[serde(default, rename = "signingTimeUtc")]
    signing_time_utc: String,
    #[serde(default, rename = "signerCertificatePath")]
    signer_certificate_path: String,
    #[serde(default, rename = "certificateChainPath")]
    certificate_chain_path: String,
    #[serde(default, rename = "trustRootsDigest")]
    trust_roots_digest: String,
    #[serde(default, rename = "keySource")]
    key_source: String,
    root: String,
    events: Vec<LogEvent>,
}

#[derive(Debug, Deserialize)]
struct LogEvent {
    seq: u64,
    #[serde(rename = "tickMs")]
    tick_ms: u64,
    #[serde(rename = "eventType")]
    event_type: String,
    #[serde(rename = "payloadDigest")]
    payload_digest: String,
    #[serde(rename = "prevHash")]
    prev_hash: String,
    hash: String,
}

#[derive(Debug, Deserialize)]
struct ReplayRecipe {
    #[serde(rename = "recipeVersion")]
    recipe_version: String,
    profile: String,
    #[serde(rename = "seedsDigest")]
    seeds_digest: String,
    #[serde(rename = "buildDigest")]
    build_digest: String,
    #[serde(rename = "configDigest")]
    config_digest: String,
    #[serde(rename = "environmentDigest")]
    environment_digest: String,
    #[serde(rename = "containerDigest")]
    container_digest: String,
    #[serde(rename = "reproducibilityGuarantee")]
    reproducibility_guarantee: String,
    #[serde(rename = "replaySlaHours")]
    replay_sla_hours: u64,
}

#[derive(Debug, Deserialize)]
struct AttestationEvidence {
    #[serde(default, rename = "attestationVersion")]
    attestation_version: String,
    level: String,
    issuer: String,
    #[serde(rename = "chainDigest")]
    chain_digest: String,
    #[serde(rename = "evidenceDigest")]
    evidence_digest: String,
    #[serde(default, rename = "evidenceSignature")]
    evidence_signature: String,
    #[serde(default, rename = "evidenceSignatureAlgorithm")]
    evidence_signature_algorithm: String,
    #[serde(default, rename = "signingTimeUtc")]
    signing_time_utc: String,
    #[serde(default, rename = "signerCertificatePath")]
    signer_certificate_path: String,
    #[serde(default, rename = "certificateChainPath")]
    certificate_chain_path: String,
    #[serde(default, rename = "trustRootsDigest")]
    trust_roots_digest: String,
    #[serde(default, rename = "revocationSnapshotDigest")]
    revocation_snapshot_digest: String,
    #[serde(default, rename = "revocationSnapshotPath")]
    revocation_snapshot_path: String,
    #[serde(default, rename = "keySource")]
    key_source: String,
    nonce: String,
    #[serde(rename = "certificateProfile")]
    certificate_profile: String,
    valid: bool,
    #[serde(rename = "keyId")]
    key_id: String,
    #[serde(rename = "deviceIdentity")]
    device_identity: String,
    #[serde(rename = "keyLifecycle")]
    key_lifecycle: KeyLifecycle,
    claims: AttestationClaims,
}

#[derive(Debug, Deserialize)]
struct KeyLifecycle {
    #[serde(rename = "rotationDays")]
    rotation_days: u64,
    #[serde(rename = "maxKeyAgeDays")]
    max_key_age_days: u64,
    #[serde(rename = "revocationEndpoint")]
    revocation_endpoint: String,
}

#[derive(Debug, Deserialize)]
struct AttestationClaims {
    #[serde(rename = "secureBootMeasured")]
    secure_boot_measured: bool,
    #[serde(rename = "hsmBacked")]
    hsm_backed: bool,
    #[serde(default, rename = "softwareStateDigest")]
    software_state_digest: String,
}

#[derive(Debug, Deserialize)]
struct AttestationTrustPolicy {
    #[serde(rename = "allowedIssuers")]
    allowed_issuers: Vec<String>,
    #[serde(rename = "allowedChainDigests")]
    allowed_chain_digests: Vec<String>,
    #[serde(rename = "requiredRevocationEndpointPrefix")]
    required_revocation_endpoint_prefix: String,
    #[serde(rename = "requiredNoncePrefix")]
    required_nonce_prefix: String,
    #[serde(rename = "minimumNonceLength")]
    minimum_nonce_length: usize,
    #[serde(default, rename = "requiredKeyUsage")]
    required_key_usage: Vec<String>,
    #[serde(default, rename = "allowedSignerCertificateDigests")]
    allowed_signer_certificate_digests: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct IncidentTemplate {
    #[serde(rename = "reportingWindowsDays")]
    reporting_windows_days: ReportingWindows,
}

#[derive(Debug, Deserialize)]
struct IncidentPackTemplate {
    #[serde(rename = "fullPackSlaDays")]
    full_pack_sla_days: u64,
}

#[derive(Debug, Deserialize)]
struct ReportingWindows {
    standard: u64,
    widespread: u64,
    fatal: u64,
}

#[derive(Debug, Deserialize)]
struct TransparencyProof {
    #[serde(default, rename = "proofVersion")]
    proof_version: String,
    #[serde(rename = "logId")]
    log_id: String,
    #[serde(default, rename = "logUrl")]
    log_url: String,
    #[serde(default, rename = "entryUuid")]
    entry_uuid: String,
    #[serde(default, rename = "logIndex")]
    log_index: u64,
    #[serde(default, rename = "treeSize")]
    tree_size: u64,
    #[serde(default, rename = "rootHash")]
    root_hash: String,
    #[serde(default, rename = "leafHash")]
    leaf_hash: String,
    #[serde(default)]
    checkpoint: String,
    #[serde(rename = "checkpointHash")]
    checkpoint_hash: String,
    #[serde(rename = "entryDigest")]
    entry_digest: String,
    #[serde(default, rename = "inclusionPath")]
    inclusion_path: Vec<String>,
    #[serde(default, rename = "consistencyPath")]
    consistency_path: Vec<String>,
    #[serde(rename = "inclusionProofHash")]
    inclusion_proof_hash: String,
    #[serde(rename = "consistencyProofHash")]
    consistency_proof_hash: String,
    #[serde(default, rename = "integratedTimeUtc")]
    integrated_time_utc: String,
    signature: String,
    #[serde(rename = "verifiedOffline")]
    verified_offline: bool,
    #[serde(default)]
    source: String,
}

#[derive(Debug, Deserialize)]
struct TrustedLogs {
    logs: Vec<TrustedLog>,
}

#[derive(Debug, Deserialize)]
struct TrustedLog {
    id: String,
    #[serde(default, rename = "checkpointHash")]
    checkpoint_hash: String,
    #[serde(default)]
    endpoint: String,
    #[serde(default, rename = "logPublicKeyPath")]
    log_public_key_path: String,
    #[serde(default, rename = "signerCertificatePath")]
    signer_certificate_path: String,
    #[serde(default, rename = "certificateChainPath")]
    certificate_chain_path: String,
    #[serde(default, rename = "trustRootsPath")]
    trust_roots_path: String,
    #[serde(default, rename = "signatureAlgorithm")]
    signature_algorithm: String,
    #[serde(default, rename = "signatureEncoding")]
    signature_encoding: String,
}

#[derive(Debug, Deserialize)]
struct BadgeRegistry {
    entries: Vec<BadgeEntry>,
}

#[derive(Debug, Deserialize)]
struct BadgeEntry {
    #[serde(rename = "badgeId")]
    badge_id: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct RevocationSnapshot {
    #[serde(default)]
    version: String,
    #[serde(default, rename = "snapshotId")]
    snapshot_id: String,
    #[serde(rename = "generatedAtUtc")]
    generated_at_utc: String,
    #[serde(rename = "nextUpdateUtc")]
    next_update_utc: String,
    #[serde(rename = "revokedSerials")]
    revoked_serials: Vec<String>,
    #[serde(default, rename = "sourceDigests")]
    source_digests: HashMap<String, String>,
    #[serde(default, rename = "ocspStatuses")]
    ocsp_statuses: Vec<OcspStatus>,
    #[serde(default)]
    signature: String,
    #[serde(default, rename = "signatureAlgorithm")]
    signature_algorithm: String,
    #[serde(default, rename = "signerCertPath")]
    signer_cert_path: String,
}

#[derive(Debug, Deserialize)]
struct OcspStatus {
    #[serde(default, rename = "sourceId")]
    source_id: String,
    #[serde(default, rename = "certSerial")]
    cert_serial: String,
    #[serde(default)]
    status: String,
    #[serde(default, rename = "checkedAtUtc")]
    checked_at_utc: String,
}

#[derive(Debug, Deserialize)]
struct FreshnessPolicy {
    #[serde(rename = "max_age_hours")]
    max_age_hours: HashMap<String, u64>,
}

#[derive(Debug, Deserialize)]
struct TransparencyVerificationPolicy {
    #[serde(default)]
    required_logs: Vec<String>,
    #[serde(default, rename = "max_checkpoint_age_hours")]
    max_checkpoint_age_hours: u64,
    #[serde(default, rename = "require_entry_digest_matches_bundle")]
    require_entry_digest_matches_bundle: bool,
    #[serde(default, rename = "require_mirror_parity")]
    require_mirror_parity: bool,
    #[serde(default, rename = "require_live_publication")]
    require_live_publication: bool,
    #[serde(default, rename = "require_checkpoint_text")]
    require_checkpoint_text: bool,
}

#[derive(Debug)]
struct VerifyContext<'a> {
    args: &'a VerifyArgs,
    trusted_map: &'a HashMap<String, TrustedLog>,
    registry: &'a BadgeRegistry,
    attestation_policy: &'a AttestationTrustPolicy,
    revocation_snapshot: &'a RevocationSnapshot,
    freshness_policy: &'a FreshnessPolicy,
    transparency_policy: &'a TransparencyVerificationPolicy,
    required_transparency: &'a [String],
    bundle_dir: &'a Path,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Verify(args) => cmd_verify(*args),
        Commands::Report(args) => cmd_report(args),
        Commands::CheckProof(args) => cmd_check_proof(args),
    }
}

fn cmd_verify(args: VerifyArgs) -> Result<()> {
    let checks: CheckCatalog =
        read_yaml(&args.checks_file).with_context(|| "failed to load check catalog")?;
    let bundle_bytes = fs::read(&args.bundle)
        .with_context(|| format!("failed reading bundle {}", args.bundle.display()))?;
    let bundle: Value =
        serde_json::from_slice(&bundle_bytes).with_context(|| "invalid bundle JSON")?;

    let required_transparency = parse_csv_list(&args.require_transparency);
    let bundle_digest = format!("sha256:{}", sha256_hex_bytes(&bundle_bytes));

    let trusted_logs: TrustedLogs = read_json(&args.trusted_checkpoints)?;
    let trusted_map = trusted_logs
        .logs
        .into_iter()
        .map(|log| (log.id.clone(), log))
        .collect::<HashMap<_, _>>();
    let registry: BadgeRegistry = read_json(&args.badge_registry)?;
    let attestation_policy: AttestationTrustPolicy = read_json(&args.attestation_trust_policy)?;
    let revocation_snapshot: RevocationSnapshot = read_json(&args.revocation_snapshot)?;
    let freshness_policy: FreshnessPolicy = read_yaml(&args.freshness_policy)?;
    let transparency_policy: TransparencyVerificationPolicy = read_yaml(&args.transparency_policy)?;
    let remediation: RemediationCatalog = read_yaml(&args.remediation_file)?;
    let bundle_dir = args.bundle.parent().unwrap_or(Path::new("."));
    let verify_ctx = VerifyContext {
        args: &args,
        trusted_map: &trusted_map,
        registry: &registry,
        attestation_policy: &attestation_policy,
        revocation_snapshot: &revocation_snapshot,
        freshness_policy: &freshness_policy,
        transparency_policy: &transparency_policy,
        required_transparency: &required_transparency,
        bundle_dir,
    };

    let mut results = Vec::new();
    for spec in checks.checks {
        let (ok, message) = evaluate_check(&spec.id, &bundle, &verify_ctx);
        let message = if let Some(hint) = remediation.checks.get(&spec.id) {
            format!("{} | remediation: {}", message, hint)
        } else {
            message
        };
        results.push(CheckResult {
            id: spec.id,
            result: if ok { "PASS".into() } else { "FAIL".into() },
            message,
        });
    }

    let failed_checks = results
        .iter()
        .filter(|c| c.result == "FAIL")
        .map(|c| c.id.clone())
        .collect::<Vec<_>>();
    let status = if failed_checks.is_empty() {
        "PASS"
    } else {
        "FAIL"
    };
    let spec_version = bundle
        .get("specVersion")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();

    let report = VerifyReport {
        tasc_verify_version: env!("CARGO_PKG_VERSION").to_string(),
        spec_version,
        profile: args.profile,
        policy: args.policy,
        bundle_digest,
        require_ta: args.require_ta,
        require_transparency: required_transparency,
        result: status.to_string(),
        checks: results,
        failed_checks,
    };

    let serialized = serde_json::to_string_pretty(&report)?;
    if let Some(output) = args.output {
        fs::write(&output, format!("{}\n", serialized))
            .with_context(|| format!("failed writing report {}", output.display()))?;
    }
    println!("{}", serialized);

    if report.result == "FAIL" {
        bail!("verification failed")
    }
    Ok(())
}

fn cmd_report(args: ReportArgs) -> Result<()> {
    let report: VerifyReport = read_json(&args.input)?;
    println!("result: {}", report.result);
    println!("profile: {}", report.profile);
    println!("policy: {}", report.policy);
    println!("checks: {}", report.checks.len());
    if !report.failed_checks.is_empty() {
        println!("failed checks:");
        for id in &report.failed_checks {
            if let Some(check) = report.checks.iter().find(|check| &check.id == id) {
                println!("- {}: {}", id, check.message);
            } else {
                println!("- {}", id);
            }
        }
    }
    Ok(())
}

fn cmd_check_proof(args: CheckProofArgs) -> Result<()> {
    let proof: TransparencyProof = read_json(&args.proof)?;
    let trusted_logs: TrustedLogs = read_json(&args.trusted_checkpoints)?;
    let trusted_map = trusted_logs
        .logs
        .into_iter()
        .map(|log| (log.id.clone(), log))
        .collect::<HashMap<_, _>>();

    let policy = TransparencyVerificationPolicy {
        required_logs: vec!["rekor".into(), "mirror".into()],
        max_checkpoint_age_hours: 0,
        require_entry_digest_matches_bundle: false,
        require_mirror_parity: false,
        require_live_publication: false,
        require_checkpoint_text: false,
    };
    let freshness = FreshnessPolicy {
        max_age_hours: HashMap::new(),
    };
    let trusted_base = args
        .trusted_checkpoints
        .parent()
        .unwrap_or(Path::new("."));
    let (ok, message) = verify_transparency_proof(
        &proof,
        &trusted_map,
        &proof.log_id,
        &policy,
        None,
        &freshness,
        trusted_base,
    );
    let result = serde_json::json!({
        "result": if ok { "PASS" } else { "FAIL" },
        "message": message,
        "logId": proof.log_id
    });
    println!("{}", serde_json::to_string_pretty(&result)?);

    if !ok {
        bail!("proof check failed")
    }
    Ok(())
}

fn evaluate_check(id: &str, bundle: &Value, ctx: &VerifyContext<'_>) -> (bool, String) {
    match id {
        "CHK_SCHEMA_ASSURANCEPACK" => check_required_paths(
            bundle,
            &[
                "/assurancePackVersion",
                "/profile",
                "/policy",
                "/specVersion",
                "/evidenceMap",
                "/conformanceReport",
                "/signedOperationalLog",
                "/replayRecipe",
                "/attestationEvidence",
                "/incidentInitialTemplate",
                "/incidentPackTemplate",
                "/transparencyProofs/rekor",
                "/transparencyProofs/mirror",
                "/badgeEntry",
                "/artifacts",
            ],
            "assurance pack envelope",
        ),
        "CHK_SCHEMA_EVIDENCEMAP" => check_required_paths(
            bundle,
            &[
                "/evidenceMap/evidenceMapVersion",
                "/evidenceMap/system",
                "/evidenceMap/tasc",
                "/evidenceMap/topology",
                "/evidenceMap/logs",
                "/evidenceMap/replay",
                "/evidenceMap/attestation",
                "/evidenceMap/incident",
            ],
            "evidence map",
        ),
        "CHK_SCHEMA_CONFORMANCE_REPORT" => check_required_paths(
            bundle,
            &[
                "/conformanceReport/tascVerifyVersion",
                "/conformanceReport/specVersion",
                "/conformanceReport/result",
                "/conformanceReport/checks",
            ],
            "conformance report",
        ),
        "CHK_SCHEMA_SIGNED_LOG" => check_required_paths(
            bundle,
            &[
                "/signedOperationalLog/schemaVersion",
                "/signedOperationalLog/signerKeyId",
                "/signedOperationalLog/signature",
                "/signedOperationalLog/root",
                "/signedOperationalLog/events",
            ],
            "signed operational log",
        ),
        "CHK_SCHEMA_REPLAY_RECIPE" => check_required_paths(
            bundle,
            &[
                "/replayRecipe/recipeVersion",
                "/replayRecipe/seedsDigest",
                "/replayRecipe/buildDigest",
                "/replayRecipe/configDigest",
                "/replayRecipe/environmentDigest",
                "/replayRecipe/containerDigest",
            ],
            "replay recipe",
        ),
        "CHK_SCHEMA_ATTESTATION" => check_required_paths(
            bundle,
            &[
                "/attestationEvidence/level",
                "/attestationEvidence/certificateProfile",
                "/attestationEvidence/keyLifecycle/rotationDays",
                "/attestationEvidence/claims/hsmBacked",
            ],
            "attestation evidence",
        ),
        "CHK_SCHEMA_INCIDENT_INITIAL" => check_required_paths(
            bundle,
            &[
                "/incidentInitialTemplate/templateId",
                "/incidentInitialTemplate/reportingWindowsDays/standard",
                "/incidentInitialTemplate/reportingWindowsDays/widespread",
                "/incidentInitialTemplate/reportingWindowsDays/fatal",
            ],
            "incident initial template",
        ),
        "CHK_SCHEMA_INCIDENT_PACK" => check_required_paths(
            bundle,
            &[
                "/incidentPackTemplate/templateId",
                "/incidentPackTemplate/requiredArtifacts",
                "/incidentPackTemplate/fullPackSlaDays",
            ],
            "incident full pack template",
        ),
        "CHK_SCHEMA_TRANSPARENCY_REKOR" => check_required_paths(
            bundle,
            &[
                "/transparencyProofs/rekor/logId",
                "/transparencyProofs/rekor/logUrl",
                "/transparencyProofs/rekor/entryUuid",
                "/transparencyProofs/rekor/logIndex",
                "/transparencyProofs/rekor/treeSize",
                "/transparencyProofs/rekor/rootHash",
                "/transparencyProofs/rekor/leafHash",
                "/transparencyProofs/rekor/checkpoint",
                "/transparencyProofs/rekor/checkpointHash",
                "/transparencyProofs/rekor/inclusionProofHash",
                "/transparencyProofs/rekor/consistencyProofHash",
                "/transparencyProofs/rekor/inclusionPath",
                "/transparencyProofs/rekor/consistencyPath",
            ],
            "rekor transparency proof",
        ),
        "CHK_SCHEMA_TRANSPARENCY_MIRROR" => check_required_paths(
            bundle,
            &[
                "/transparencyProofs/mirror/logId",
                "/transparencyProofs/mirror/logUrl",
                "/transparencyProofs/mirror/entryUuid",
                "/transparencyProofs/mirror/logIndex",
                "/transparencyProofs/mirror/treeSize",
                "/transparencyProofs/mirror/rootHash",
                "/transparencyProofs/mirror/leafHash",
                "/transparencyProofs/mirror/checkpoint",
                "/transparencyProofs/mirror/checkpointHash",
                "/transparencyProofs/mirror/inclusionProofHash",
                "/transparencyProofs/mirror/consistencyProofHash",
                "/transparencyProofs/mirror/inclusionPath",
                "/transparencyProofs/mirror/consistencyPath",
            ],
            "mirror transparency proof",
        ),
        "CHK_SCHEMA_BADGE_ENTRY" => check_required_paths(
            bundle,
            &[
                "/badgeEntry/badgeId",
                "/badgeEntry/status",
                "/badgeEntry/badgeType",
            ],
            "badge entry",
        ),
        "CHK_PROFILE_MATCH" => check_bundle_value(bundle, "/profile", &ctx.args.profile, "profile"),
        "CHK_POLICY_MATCH" => check_bundle_value(bundle, "/policy", &ctx.args.policy, "policy"),
        "CHK_TOPOLOGY_INTERLOCK_MEDIATION" => check_interlock_mediation(bundle),
        "CHK_TOPOLOGY_AUTHORITY_EXCLUSIVITY" => check_authority_exclusivity(bundle),
        "CHK_TOPOLOGY_ASSERTION_REFS" => check_assertion_refs(bundle),
        "CHK_ARTIFACT_HASH_INTEGRITY" => check_artifact_hashes(bundle, ctx.bundle_dir),
        "CHK_LOG_HASH_CHAIN" => check_log_hash_chain(bundle),
        "CHK_LOG_SIGNATURE" => check_log_signature(bundle, ctx),
        "CHK_RETENTION_EU_MINIMUM" => check_retention_minimum(bundle, 6),
        "CHK_REPLAY_COMPLETENESS" => check_replay(bundle, &ctx.args.profile),
        "CHK_REPLAY_OPERATIONAL_PARITY" => {
            check_replay_operational_parity(bundle, &ctx.args.profile, ctx.bundle_dir)
        }
        "CHK_ATTESTATION_TA2" => check_attestation(bundle, ctx),
        "CHK_INCIDENT_WINDOWS_EU" => check_incident_windows(bundle),
        "CHK_TRANSPARENCY_REKOR_PROOF" => {
            check_named_proof(
                bundle,
                ctx.trusted_map,
                ctx.required_transparency,
                "rekor",
                ctx.transparency_policy,
                ctx.freshness_policy,
                ctx.bundle_dir,
            )
        }
        "CHK_TRANSPARENCY_MIRROR_PROOF" => {
            check_named_proof(
                bundle,
                ctx.trusted_map,
                ctx.required_transparency,
                "mirror",
                ctx.transparency_policy,
                ctx.freshness_policy,
                ctx.bundle_dir,
            )
        }
        "CHK_BADGE_ACTIVE_NOT_REVOKED" => check_badge_status(bundle, ctx.registry),
        _ => (false, format!("unknown check id {}", id)),
    }
}

fn check_required_paths(bundle: &Value, paths: &[&str], label: &str) -> (bool, String) {
    let missing = paths
        .iter()
        .copied()
        .filter(|path| bundle.pointer(path).is_none())
        .collect::<Vec<_>>();
    if missing.is_empty() {
        (true, format!("{} present", label))
    } else {
        (
            false,
            format!("missing {} fields: {}", label, missing.join(", ")),
        )
    }
}

fn check_bundle_value(
    bundle: &Value,
    pointer: &str,
    expected: &str,
    label: &str,
) -> (bool, String) {
    let value = bundle.pointer(pointer).and_then(Value::as_str);
    match value {
        Some(found) if found == expected => (true, format!("{} matches {}", label, expected)),
        Some(found) => (
            false,
            format!("{} mismatch: expected {}, got {}", label, expected, found),
        ),
        None => (false, format!("{} missing at {}", label, pointer)),
    }
}

fn check_interlock_mediation(bundle: &Value) -> (bool, String) {
    let nodes = match topology_nodes(bundle) {
        Ok(v) => v,
        Err(e) => return (false, e.to_string()),
    };
    let edges = match topology_edges(bundle) {
        Ok(v) => v,
        Err(e) => return (false, e.to_string()),
    };

    let node_types = nodes
        .iter()
        .map(|n| (n.id.as_str(), n.node_type.as_str()))
        .collect::<HashMap<_, _>>();

    let mut actuator_edges = 0usize;
    for edge in edges {
        let to_type = node_types
            .get(edge.to.as_str())
            .copied()
            .unwrap_or("unknown");
        if to_type == "actuator_integrator" {
            actuator_edges += 1;
            let from_type = node_types
                .get(edge.from.as_str())
                .copied()
                .unwrap_or("unknown");
            if from_type != "interlock_gate" {
                return (
                    false,
                    format!(
                        "actuator edge {} -> {} not mediated by interlock_gate",
                        edge.from, edge.to
                    ),
                );
            }
        }
    }

    if actuator_edges == 0 {
        return (false, "no actuator_integrator edges present".into());
    }
    (true, "all actuator paths mediated by interlock gate".into())
}

fn check_authority_exclusivity(bundle: &Value) -> (bool, String) {
    let nodes = match topology_nodes(bundle) {
        Ok(v) => v,
        Err(e) => return (false, e.to_string()),
    };
    let edges = match topology_edges(bundle) {
        Ok(v) => v,
        Err(e) => return (false, e.to_string()),
    };

    let node_types = nodes
        .iter()
        .map(|n| (n.id.as_str(), n.node_type.as_str()))
        .collect::<HashMap<_, _>>();

    let mut authority_edges = 0usize;
    for edge in edges {
        let _ = &edge.signal;
        let to_type = node_types
            .get(edge.to.as_str())
            .copied()
            .unwrap_or("unknown");
        if to_type == "interlock_gate" {
            authority_edges += 1;
            let from_type = node_types
                .get(edge.from.as_str())
                .copied()
                .unwrap_or("unknown");
            if from_type != "safety_kernel" {
                return (
                    false,
                    format!(
                        "non-kernel source {} targets interlock gate {}",
                        edge.from, edge.to
                    ),
                );
            }
        }
    }

    if authority_edges == 0 {
        return (false, "no interlock authority edges present".into());
    }
    (
        true,
        "interlock authority exclusive to safety kernel".into(),
    )
}

fn check_assertion_refs(bundle: &Value) -> (bool, String) {
    let assertions_val = match bundle.pointer("/evidenceMap/topology/conformanceAssertions") {
        Some(v) => v.clone(),
        None => return (false, "missing conformanceAssertions".into()),
    };
    let assertions: Vec<TopologyAssertion> = match serde_json::from_value(assertions_val) {
        Ok(v) => v,
        Err(e) => return (false, format!("invalid conformanceAssertions: {}", e)),
    };

    for required in ["A001", "A002"] {
        let Some(found) = assertions.iter().find(|a| a.id == required) else {
            return (false, format!("required assertion {} missing", required));
        };
        if found.status != "proven" {
            return (false, format!("assertion {} not proven", required));
        }
        if !is_sha_prefixed(&found.proof_ref) {
            return (false, format!("assertion {} proofRef invalid", required));
        }
    }

    (
        true,
        "required assertion refs are proven and hash-addressed".into(),
    )
}

fn check_artifact_hashes(bundle: &Value, bundle_dir: &Path) -> (bool, String) {
    let artifacts_val = match bundle.get("artifacts") {
        Some(v) => v.clone(),
        None => return (false, "missing artifacts list".into()),
    };
    let artifacts: Vec<ArtifactRef> = match serde_json::from_value(artifacts_val) {
        Ok(v) => v,
        Err(e) => return (false, format!("invalid artifacts list: {}", e)),
    };

    if artifacts.is_empty() {
        return (false, "artifacts list must not be empty".into());
    }

    for artifact in artifacts {
        let path = resolve_artifact_path(&artifact.path, bundle_dir);
        if !path.exists() {
            return (false, format!("missing artifact file {}", path.display()));
        }
        let digest = match sha256_file_prefixed(&path) {
            Ok(v) => v,
            Err(e) => return (false, format!("failed hashing {}: {}", path.display(), e)),
        };
        if digest != artifact.sha256 {
            return (
                false,
                format!(
                    "artifact digest mismatch for {} (expected {}, got {})",
                    artifact.path, artifact.sha256, digest
                ),
            );
        }
    }

    (true, "artifact hashes validated".into())
}

fn check_log_hash_chain(bundle: &Value) -> (bool, String) {
    let log_val = match bundle.get("signedOperationalLog") {
        Some(v) => v.clone(),
        None => return (false, "missing signedOperationalLog".into()),
    };
    let log: SignedLog = match serde_json::from_value(log_val) {
        Ok(v) => v,
        Err(e) => return (false, format!("invalid signedOperationalLog: {}", e)),
    };

    if log.events.is_empty() {
        return (false, "log events are empty".into());
    }

    let mut last_hash = String::new();
    for (idx, event) in log.events.iter().enumerate() {
        if idx == 0 {
            if !event.prev_hash.is_empty() {
                return (false, "first event prevHash must be empty".into());
            }
        } else if event.prev_hash != last_hash {
            return (
                false,
                format!("event {} prevHash does not match prior hash", idx),
            );
        }

        let expected = hash_log_event(event);
        if expected != event.hash {
            return (
                false,
                format!(
                    "event {} hash mismatch (expected {}, got {})",
                    idx, expected, event.hash
                ),
            );
        }
        last_hash = event.hash.clone();
    }

    if log.root != last_hash {
        return (
            false,
            format!(
                "log root mismatch (expected {}, got {})",
                last_hash, log.root
            ),
        );
    }

    (true, "log hash chain verified".into())
}

fn check_log_signature(bundle: &Value, ctx: &VerifyContext<'_>) -> (bool, String) {
    let log_val = match bundle.get("signedOperationalLog") {
        Some(v) => v.clone(),
        None => return (false, "missing signedOperationalLog".into()),
    };
    let log: SignedLog = match serde_json::from_value(log_val) {
        Ok(v) => v,
        Err(e) => return (false, format!("invalid signedOperationalLog: {}", e)),
    };

    if log.schema_version != "0.2" {
        return (
            false,
            format!("unsupported signedOperationalLog schemaVersion {}", log.schema_version),
        );
    }
    if log.signer_key_id.trim().is_empty() {
        return (false, "signerKeyId is required for schemaVersion 0.2".into());
    }
    if log.signature_algorithm != "rsa-sha256" || log.signature_encoding != "base64" {
        return (
            false,
            "0.2 log signature must use rsa-sha256/base64".into(),
        );
    }
    if log.signing_time_utc.trim().is_empty() {
        return (false, "signingTimeUtc is required for schemaVersion 0.2".into());
    }
    let signer_cert = resolve_artifact_path(&log.signer_certificate_path, ctx.bundle_dir);
    let chain = resolve_artifact_path(&log.certificate_chain_path, ctx.bundle_dir);
    let max_age = ctx
        .freshness_policy
        .max_age_hours
        .get("assurance_pack")
        .copied()
        .unwrap_or(168);
    if let Err(err) = check_file_age_hours(signer_cert.clone(), max_age, "signer certificate freshness")
    {
        return (false, err.to_string());
    }
    let trust_roots = resolve_policy_path(&ctx.args.trust_roots, ctx.bundle_dir);
    if !signer_cert.exists() || !chain.exists() || !trust_roots.exists() {
        return (false, "missing cert/chain/trust roots for log signature".into());
    }
    let expected_trust_digest = match sha256_file_prefixed(&trust_roots) {
        Ok(v) => v,
        Err(e) => return (false, format!("failed to hash trust roots: {}", e)),
    };
    if expected_trust_digest != log.trust_roots_digest {
        return (
            false,
            format!(
                "trustRootsDigest mismatch (expected {}, got {})",
                expected_trust_digest, log.trust_roots_digest
            ),
        );
    }

    if let Err(err) = verify_certificate_chain(&signer_cert, &chain, &trust_roots) {
        return (false, format!("certificate chain verification failed: {}", err));
    }
    if let Err(err) = verify_signature_base64(&log.root, &log.signature, &signer_cert) {
        return (false, format!("log signature verification failed: {}", err));
    }
    if ctx.args.require_ta == "TA2" && log.key_source != "PKCS11" {
        return (
            false,
            format!("TA2 log signature requires keySource=PKCS11, got {}", log.key_source),
        );
    }

    (
        true,
        format!(
            "log signature validated with {} key source",
            if log.key_source.is_empty() {
                "unknown"
            } else {
                &log.key_source
            }
        ),
    )
}

fn check_retention_minimum(bundle: &Value, min_months: u64) -> (bool, String) {
    let months = bundle
        .pointer("/evidenceMap/logs/retentionPolicy/minimumMonths")
        .and_then(Value::as_u64);
    match months {
        Some(value) if value >= min_months => (
            true,
            format!("retention minimum satisfied ({} months)", value),
        ),
        Some(value) => (
            false,
            format!(
                "retention minimum {} months below required {}",
                value, min_months
            ),
        ),
        None => (false, "missing retention minimum".into()),
    }
}

fn check_replay(bundle: &Value, expected_profile: &str) -> (bool, String) {
    let replay_val = match bundle.get("replayRecipe") {
        Some(v) => v.clone(),
        None => return (false, "missing replayRecipe".into()),
    };
    let replay: ReplayRecipe = match serde_json::from_value(replay_val) {
        Ok(v) => v,
        Err(e) => return (false, format!("invalid replayRecipe: {}", e)),
    };

    let fields = [
        replay.seeds_digest,
        replay.build_digest,
        replay.config_digest,
        replay.environment_digest,
        replay.container_digest,
    ];
    if fields.iter().any(|value| !is_sha_prefixed(value)) {
        return (false, "one or more replay digests are invalid".into());
    }
    if replay.replay_sla_hours == 0 {
        return (false, "replaySlaHours must be > 0".into());
    }
    if replay.reproducibility_guarantee.trim().is_empty() {
        return (false, "reproducibilityGuarantee missing".into());
    }
    if replay.recipe_version != "0.1" {
        return (
            false,
            format!(
                "unsupported replay recipe version {}",
                replay.recipe_version
            ),
        );
    }
    if replay.profile != expected_profile {
        return (
            false,
            format!(
                "replay profile {} != expected {}",
                replay.profile, expected_profile
            ),
        );
    }
    (true, "replay completeness checks passed".into())
}

fn check_replay_operational_parity(
    bundle: &Value,
    expected_profile: &str,
    bundle_dir: &Path,
) -> (bool, String) {
    let artifacts: Vec<ArtifactRef> = match bundle.get("artifacts").cloned() {
        Some(v) => match serde_json::from_value(v) {
            Ok(parsed) => parsed,
            Err(e) => return (false, format!("invalid artifacts list: {}", e)),
        },
        None => return (false, "missing artifacts list".into()),
    };

    let replay_artifact = artifacts.iter().find(|artifact| {
        artifact.path.contains("replay-from-log-") && artifact.path.ends_with(".json")
    });
    let Some(replay_artifact) = replay_artifact else {
        return (
            false,
            "missing replay-from-log report artifact for operational parity".into(),
        );
    };

    let replay_path = resolve_artifact_path(&replay_artifact.path, bundle_dir);
    let replay_payload: Value = match read_json(&replay_path) {
        Ok(v) => v,
        Err(e) => {
            return (
                false,
                format!(
                    "failed reading replay-from-log report {}: {}",
                    replay_path.display(),
                    e
                ),
            )
        }
    };
    let result = replay_payload
        .get("result")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if result != "PASS" {
        return (
            false,
            format!(
                "replay-from-log report result is {}, expected PASS",
                result
            ),
        );
    }
    let drift_count = replay_payload
        .get("drift_count")
        .and_then(Value::as_u64)
        .unwrap_or(1);
    if drift_count != 0 {
        return (
            false,
            format!("replay-from-log drift_count must be 0, got {}", drift_count),
        );
    }
    let report_profile = replay_payload
        .get("profile")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if report_profile != expected_profile {
        return (
            false,
            format!(
                "replay-from-log profile {} != expected {}",
                report_profile, expected_profile
            ),
        );
    }

    (
        true,
        format!(
            "replay-from-log operational parity verified for {}",
            expected_profile
        ),
    )
}

fn check_attestation(bundle: &Value, ctx: &VerifyContext<'_>) -> (bool, String) {
    let att_val = match bundle.get("attestationEvidence") {
        Some(v) => v.clone(),
        None => return (false, "missing attestationEvidence".into()),
    };
    let att: AttestationEvidence = match serde_json::from_value(att_val) {
        Ok(v) => v,
        Err(e) => return (false, format!("invalid attestationEvidence: {}", e)),
    };

    let level_rank = |level: &str| match level {
        "TA0" => Some(0),
        "TA1" => Some(1),
        "TA2" => Some(2),
        _ => None,
    };

    let Some(found_rank) = level_rank(&att.level) else {
        return (false, format!("unknown attestation level {}", att.level));
    };
    let Some(required_rank) = level_rank(&ctx.args.require_ta) else {
        return (false, format!("unknown required TA level {}", ctx.args.require_ta));
    };

    if found_rank < required_rank {
        return (
            false,
            format!(
                "attestation level {} below required {}",
                att.level, ctx.args.require_ta
            ),
        );
    }

    if !att.valid {
        return (false, "attestation valid=false".into());
    }
    if att.device_identity.trim().is_empty() || att.key_id.trim().is_empty() {
        return (
            false,
            "attestation deviceIdentity/keyId must be present".into(),
        );
    }
    if !is_sha_prefixed(&att.chain_digest) {
        return (
            false,
            "attestation chainDigest must be sha256-prefixed".into(),
        );
    }
    let att_issuer_normalized = att.issuer.replace(' ', "");
    if !ctx
        .attestation_policy
        .allowed_issuers
        .iter()
        .any(|issuer| issuer.replace(' ', "") == att_issuer_normalized)
    {
        return (
            false,
            format!("attestation issuer {} is not trusted", att.issuer),
        );
    }
    if !ctx
        .attestation_policy
        .allowed_chain_digests
        .iter()
        .any(|digest| digest == &att.chain_digest)
    {
        return (
            false,
            format!(
                "attestation chainDigest {} is not trusted",
                att.chain_digest
            ),
        );
    }
    if att.nonce.len() < ctx.attestation_policy.minimum_nonce_length
        || !att.nonce.starts_with(&ctx.attestation_policy.required_nonce_prefix)
    {
        return (
            false,
            format!(
                "attestation nonce must start with {} and be >= {} chars",
                ctx.attestation_policy.required_nonce_prefix, ctx.attestation_policy.minimum_nonce_length
            ),
        );
    }
    if !att
        .key_lifecycle
        .revocation_endpoint
        .starts_with(&ctx.attestation_policy.required_revocation_endpoint_prefix)
    {
        return (
            false,
            format!(
                "revocation endpoint must start with {}",
                ctx.attestation_policy.required_revocation_endpoint_prefix
            ),
        );
    }
    if att.key_lifecycle.max_key_age_days < att.key_lifecycle.rotation_days {
        return (
            false,
            format!(
                "maxKeyAgeDays {} below rotationDays {}",
                att.key_lifecycle.max_key_age_days, att.key_lifecycle.rotation_days
            ),
        );
    }

    let trust_roots = resolve_policy_path(&ctx.args.trust_roots, ctx.bundle_dir);
    let revocation_snapshot_path = resolve_policy_path(&ctx.args.revocation_snapshot, ctx.bundle_dir);
    if !trust_roots.exists() || !revocation_snapshot_path.exists() {
        return (false, "trust roots or revocation snapshot path missing".into());
    }
    let trust_roots_digest = match sha256_file_prefixed(&trust_roots) {
        Ok(v) => v,
        Err(e) => return (false, format!("failed hashing trust roots: {}", e)),
    };
    if !att.trust_roots_digest.is_empty() && att.trust_roots_digest != trust_roots_digest {
        return (
            false,
            format!(
                "attestation trustRootsDigest mismatch (expected {}, got {})",
                trust_roots_digest, att.trust_roots_digest
            ),
        );
    }
    let revocation_digest = match sha256_file_prefixed(&revocation_snapshot_path) {
        Ok(v) => v,
        Err(e) => return (false, format!("failed hashing revocation snapshot: {}", e)),
    };
    if !att.revocation_snapshot_digest.is_empty()
        && att.revocation_snapshot_digest != revocation_digest
    {
        return (
            false,
            format!(
                "attestation revocationSnapshotDigest mismatch (expected {}, got {})",
                revocation_digest, att.revocation_snapshot_digest
            ),
        );
    }
    if !att.revocation_snapshot_path.is_empty() {
        let declared_revocation_path =
            resolve_artifact_path(&att.revocation_snapshot_path, ctx.bundle_dir);
        if !declared_revocation_path.exists() {
            return (
                false,
                format!(
                    "declared revocationSnapshotPath does not exist: {}",
                    declared_revocation_path.display()
                ),
            );
        }
        let declared_digest = match sha256_file_prefixed(&declared_revocation_path) {
            Ok(v) => v,
            Err(e) => {
                return (
                    false,
                    format!("failed hashing declared revocationSnapshotPath: {}", e),
                )
            }
        };
        if declared_digest != revocation_digest {
            return (
                false,
                format!(
                    "declared revocationSnapshotPath digest mismatch (expected {}, got {})",
                    revocation_digest, declared_digest
                ),
            );
        }
    }
    if !att.key_source.is_empty() && att.key_source != "FILE" && att.key_source != "PKCS11" {
        return (
            false,
            format!("unsupported attestation keySource {}", att.key_source),
        );
    }
    if !att.attestation_version.is_empty() && att.attestation_version != "0.2" {
        return (
            false,
            format!(
                "unsupported attestationVersion {} (expected 0.2)",
                att.attestation_version
            ),
        );
    }

    let signer_cert = if att.signer_certificate_path.is_empty() {
        resolve_artifact_path("policies/attestation/pki/ta2-signer.cert.pem", ctx.bundle_dir)
    } else {
        resolve_artifact_path(&att.signer_certificate_path, ctx.bundle_dir)
    };
    let chain_path = if att.certificate_chain_path.is_empty() {
        resolve_artifact_path("policies/attestation/pki/ta2-chain.pem", ctx.bundle_dir)
    } else {
        resolve_artifact_path(&att.certificate_chain_path, ctx.bundle_dir)
    };
    if !signer_cert.exists() || !chain_path.exists() {
        return (false, "attestation signer certificate/chain missing".into());
    }

    if let Err(err) = verify_certificate_chain(&signer_cert, &chain_path, &trust_roots) {
        return (false, format!("attestation cert chain invalid: {}", err));
    }
    let snapshot_signer_cert = if ctx.revocation_snapshot.signer_cert_path.trim().is_empty() {
        signer_cert.clone()
    } else {
        resolve_artifact_path(&ctx.revocation_snapshot.signer_cert_path, ctx.bundle_dir)
    };
    if !snapshot_signer_cert.exists() {
        return (
            false,
            format!(
                "revocation snapshot signer certificate missing: {}",
                snapshot_signer_cert.display()
            ),
        );
    }
    if let Err(err) = verify_certificate_chain(&snapshot_signer_cert, &chain_path, &trust_roots) {
        return (
            false,
            format!("revocation snapshot signer cert chain invalid: {}", err),
        );
    }
    if !att.evidence_signature.is_empty() {
        if att.evidence_signature_algorithm != "rsa-sha256" {
            return (
                false,
                format!(
                    "unsupported evidenceSignatureAlgorithm {}",
                    att.evidence_signature_algorithm
                ),
            );
        }
        if let Err(err) =
            verify_signature_base64(&att.evidence_digest, &att.evidence_signature, &signer_cert)
        {
            return (false, format!("attestation signature invalid: {}", err));
        }
    }
    if !ctx.attestation_policy.allowed_signer_certificate_digests.is_empty() {
        let signer_cert_digest = match sha256_file_prefixed(&signer_cert) {
            Ok(v) => v,
            Err(e) => return (false, format!("failed hashing signer certificate: {}", e)),
        };
        if !ctx
            .attestation_policy
            .allowed_signer_certificate_digests
            .iter()
            .any(|digest| digest == &signer_cert_digest)
        {
            return (
                false,
                format!("signer certificate digest {} is not allowed", signer_cert_digest),
            );
        }
    }

    let serial = match certificate_serial_hex(&signer_cert) {
        Ok(v) => v,
        Err(e) => return (false, format!("failed to read signer cert serial: {}", e)),
    };
    if ctx
        .revocation_snapshot
        .revoked_serials
        .iter()
        .any(|revoked| revoked.eq_ignore_ascii_case(&serial))
    {
        return (false, format!("signer certificate serial {} is revoked", serial));
    }
    if ctx.revocation_snapshot.generated_at_utc.trim().is_empty()
        || ctx.revocation_snapshot.next_update_utc.trim().is_empty()
    {
        return (false, "revocation snapshot missing generatedAtUtc/nextUpdateUtc".into());
    }
    if !ctx.revocation_snapshot.version.is_empty() && ctx.revocation_snapshot.version != "0.3.0" {
        return (
            false,
            format!(
                "unsupported revocation snapshot version {}",
                ctx.revocation_snapshot.version
            ),
        );
    }
    if ctx.revocation_snapshot.snapshot_id.trim().is_empty() {
        return (false, "revocation snapshot missing snapshotId".into());
    }
    if ctx.revocation_snapshot.source_digests.is_empty() {
        return (false, "revocation snapshot sourceDigests must not be empty".into());
    }
    let mut has_crl_source = false;
    let mut has_ocsp_source = false;
    for (source_id, digest) in &ctx.revocation_snapshot.source_digests {
        if !is_sha_prefixed(digest) {
            return (
                false,
                format!("revocation source digest for {} is not sha256-prefixed", source_id),
            );
        }
        let lowered = source_id.to_lowercase();
        if lowered.contains("crl") {
            has_crl_source = true;
        }
        if lowered.contains("ocsp") {
            has_ocsp_source = true;
        }
    }
    if !has_crl_source {
        return (false, "revocation snapshot must include at least one CRL source digest".into());
    }
    if !has_ocsp_source {
        return (false, "revocation snapshot must include at least one OCSP source digest".into());
    }
    if ctx.revocation_snapshot.ocsp_statuses.is_empty() {
        return (false, "revocation snapshot ocspStatuses must not be empty".into());
    }
    for (idx, status) in ctx.revocation_snapshot.ocsp_statuses.iter().enumerate() {
        if status.source_id.trim().is_empty()
            || status.cert_serial.trim().is_empty()
            || status.checked_at_utc.trim().is_empty()
        {
            return (
                false,
                format!("ocspStatuses[{}] missing sourceId/certSerial/checkedAtUtc", idx),
            );
        }
        match status.status.as_str() {
            "good" | "revoked" | "unknown" => {}
            _ => {
                return (
                    false,
                    format!("ocspStatuses[{}] has invalid status {}", idx, status.status),
                )
            }
        }
        if let Err(err) = parse_utc(&status.checked_at_utc) {
            return (
                false,
                format!("ocspStatuses[{}] invalid checkedAtUtc: {}", idx, err),
            );
        }
    }
    if ctx.revocation_snapshot.signature_algorithm != "rsa-sha256" {
        return (
            false,
            format!(
                "unsupported revocation snapshot signatureAlgorithm {}",
                ctx.revocation_snapshot.signature_algorithm
            ),
        );
    }
    if ctx.revocation_snapshot.signature.trim().is_empty() {
        return (false, "revocation snapshot signature is required".into());
    }
    let snapshot_raw: Value = match read_json(&revocation_snapshot_path) {
        Ok(v) => v,
        Err(err) => {
            return (
                false,
                format!("failed reading revocation snapshot raw payload: {}", err),
            )
        }
    };
    let Some(snapshot_obj) = snapshot_raw.as_object() else {
        return (false, "revocation snapshot must be a JSON object".into());
    };

    let mut snapshot_id_obj = snapshot_obj.clone();
    snapshot_id_obj.remove("snapshotId");
    snapshot_id_obj.remove("signature");
    snapshot_id_obj.remove("signatureAlgorithm");
    snapshot_id_obj.remove("signerCertPath");
    let snapshot_id_payload = Value::Object(snapshot_id_obj);
    let snapshot_id_expected = format!("sha256:{}", sha256_hex_str(&canonical_json(&snapshot_id_payload)));
    if snapshot_id_expected != ctx.revocation_snapshot.snapshot_id {
        return (
            false,
            format!(
                "revocation snapshotId mismatch (expected {}, got {})",
                snapshot_id_expected, ctx.revocation_snapshot.snapshot_id
            ),
        );
    }

    let mut signature_obj = snapshot_obj.clone();
    signature_obj.remove("signature");
    signature_obj.remove("signatureAlgorithm");
    signature_obj.remove("signerCertPath");
    let signable_snapshot_canonical = canonical_json(&Value::Object(signature_obj));
    if let Err(err) = verify_signature_base64(
        &signable_snapshot_canonical,
        &ctx.revocation_snapshot.signature,
        &snapshot_signer_cert,
    ) {
        return (
            false,
            format!("revocation snapshot signature invalid: {}", err),
        );
    }
    let revocation_max_age = ctx
        .freshness_policy
        .max_age_hours
        .get("revocation_snapshot")
        .copied()
        .unwrap_or(720);
    if let Err(err) = check_file_age_hours(
        revocation_snapshot_path,
        revocation_max_age,
        "revocation snapshot freshness",
    ) {
        return (false, err.to_string());
    }
    let generated_at = match parse_utc(&ctx.revocation_snapshot.generated_at_utc) {
        Ok(v) => v,
        Err(e) => return (false, format!("invalid generatedAtUtc: {}", e)),
    };
    let next_update = match parse_utc(&ctx.revocation_snapshot.next_update_utc) {
        Ok(v) => v,
        Err(e) => return (false, format!("invalid nextUpdateUtc: {}", e)),
    };
    let now = Utc::now();
    if generated_at > next_update {
        return (false, "revocation snapshot generatedAtUtc is after nextUpdateUtc".into());
    }
    if now > next_update {
        return (false, "revocation snapshot nextUpdateUtc has expired".into());
    }
    if att.signing_time_utc.trim().is_empty() {
        return (false, "attestation signingTimeUtc is required".into());
    }
    let att_signing_time = match parse_utc(&att.signing_time_utc) {
        Ok(v) => v,
        Err(e) => return (false, format!("invalid attestation signingTimeUtc: {}", e)),
    };
    if att_signing_time > now {
        return (false, "attestation signingTimeUtc cannot be in the future".into());
    }
    if att_signing_time < generated_at {
        return (
            false,
            "attestation signingTimeUtc predates revocation snapshot generation".into(),
        );
    }
    if !ctx.attestation_policy.required_key_usage.is_empty() {
        let usage = match certificate_text(&signer_cert) {
            Ok(v) => v,
            Err(e) => return (false, format!("failed to inspect certificate usage: {}", e)),
        };
        for required_usage in &ctx.attestation_policy.required_key_usage {
            if !usage.contains(required_usage) {
                return (
                    false,
                    format!("required certificate key usage {} not present", required_usage),
                );
            }
        }
    }
    let chain_digest = match sha256_file_prefixed(&chain_path) {
        Ok(v) => v,
        Err(e) => return (false, format!("failed hashing certificate chain: {}", e)),
    };
    if chain_digest != att.chain_digest {
        return (
            false,
            format!(
                "attestation chainDigest mismatch (expected {}, got {})",
                chain_digest, att.chain_digest
            ),
        );
    }
    let expected_evidence_digest = compute_attestation_evidence_digest(bundle, &att);
    if expected_evidence_digest != att.evidence_digest {
        return (
            false,
            format!(
                "attestation evidenceDigest mismatch (expected {}, got {})",
                expected_evidence_digest, att.evidence_digest
            ),
        );
    }

    if att.level == "TA2" {
        if att.certificate_profile != "HSM_CERTIFIED"
            && att.certificate_profile != "SECURE_ELEMENT_CERTIFIED"
        {
            return (
                false,
                format!(
                    "invalid TA2 certificate profile {}",
                    att.certificate_profile
                ),
            );
        }
        if !att.claims.hsm_backed {
            return (false, "TA2 requires hsmBacked=true".into());
        }
        if !att.claims.secure_boot_measured {
            return (false, "TA2 requires secureBootMeasured=true".into());
        }
        if att.key_lifecycle.rotation_days > 365 {
            return (
                false,
                format!(
                    "TA2 rotationDays {} exceeds max 365",
                    att.key_lifecycle.rotation_days
                ),
            );
        }
        if att.key_lifecycle.max_key_age_days > 365 {
            return (
                false,
                format!(
                    "TA2 maxKeyAgeDays {} exceeds max 365",
                    att.key_lifecycle.max_key_age_days
                ),
            );
        }
    }

    if att.level == "TA2" && att.key_source != "PKCS11" {
        return (
            false,
            format!("TA2 attestation requires keySource=PKCS11, got {}", att.key_source),
        );
    }

    (
        true,
        format!(
            "attestation {} satisfies minimum {}",
            att.level, ctx.args.require_ta
        ),
    )
}

fn check_incident_windows(bundle: &Value) -> (bool, String) {
    let initial: IncidentTemplate = match bundle.get("incidentInitialTemplate").cloned() {
        Some(v) => match serde_json::from_value(v) {
            Ok(parsed) => parsed,
            Err(e) => return (false, format!("invalid incidentInitialTemplate: {}", e)),
        },
        None => return (false, "missing incidentInitialTemplate".into()),
    };

    let pack: IncidentPackTemplate = match bundle.get("incidentPackTemplate").cloned() {
        Some(v) => match serde_json::from_value(v) {
            Ok(parsed) => parsed,
            Err(e) => return (false, format!("invalid incidentPackTemplate: {}", e)),
        },
        None => return (false, "missing incidentPackTemplate".into()),
    };

    let w = initial.reporting_windows_days;
    if !(w.standard == 15 && w.widespread == 2 && w.fatal == 10) {
        return (
            false,
            format!(
                "incident windows must be 15/2/10, got {}/{}/{}",
                w.standard, w.widespread, w.fatal
            ),
        );
    }
    if pack.full_pack_sla_days > 10 {
        return (
            false,
            format!("fullPackSlaDays {} exceeds max 10", pack.full_pack_sla_days),
        );
    }

    (true, "incident window policy checks passed".into())
}

fn check_named_proof(
    bundle: &Value,
    trusted_map: &HashMap<String, TrustedLog>,
    required_transparency: &[String],
    expected: &str,
    policy: &TransparencyVerificationPolicy,
    freshness_policy: &FreshnessPolicy,
    bundle_dir: &Path,
) -> (bool, String) {
    if !policy.required_logs.is_empty() && !policy.required_logs.iter().any(|log| log == expected) {
        return (true, format!("{} not required by transparency policy", expected));
    }
    if !required_transparency.iter().any(|log| log == expected) {
        return (true, format!("{} not required by invocation", expected));
    }

    let pointer = format!("/transparencyProofs/{}", expected);
    let Some(proof_val) = bundle.pointer(&pointer).cloned() else {
        return (
            false,
            format!("missing transparency proof for {}", expected),
        );
    };
    let proof: TransparencyProof = match serde_json::from_value(proof_val) {
        Ok(v) => v,
        Err(e) => {
            return (
                false,
                format!("invalid proof payload for {}: {}", expected, e),
            )
        }
    };

    let declared_bundle_digest = bundle
        .pointer("/lineage/assurancePackDigest")
        .and_then(Value::as_str)
        .or_else(|| bundle.pointer("/conformanceReport/bundleDigest").and_then(Value::as_str));

    if expected == "mirror" && policy.require_mirror_parity {
        let rekor_digest = bundle
            .pointer("/transparencyProofs/rekor/entryDigest")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if !rekor_digest.is_empty() && rekor_digest != proof.entry_digest {
            return (
                false,
                format!(
                    "mirror parity mismatch: mirror entryDigest {} != rekor entryDigest {}",
                    proof.entry_digest, rekor_digest
                ),
            );
        }
    }

    verify_transparency_proof(
        &proof,
        trusted_map,
        expected,
        policy,
        declared_bundle_digest,
        freshness_policy,
        bundle_dir,
    )
}

fn verify_transparency_proof(
    proof: &TransparencyProof,
    trusted_map: &HashMap<String, TrustedLog>,
    expected_log: &str,
    policy: &TransparencyVerificationPolicy,
    declared_bundle_digest: Option<&str>,
    freshness_policy: &FreshnessPolicy,
    bundle_dir: &Path,
) -> (bool, String) {
    if !proof.proof_version.is_empty() && proof.proof_version != "0.1" && proof.proof_version != "0.2" {
        return (
            false,
            format!("unsupported proofVersion {} for {}", proof.proof_version, expected_log),
        );
    }
    if proof.log_id != expected_log {
        return (
            false,
            format!("proof log id {} != expected {}", proof.log_id, expected_log),
        );
    }
    if !proof.verified_offline {
        return (false, "proof must set verifiedOffline=true".into());
    }
    if proof.entry_uuid.trim().is_empty() {
        return (false, "entryUuid is required".into());
    }
    if policy.require_live_publication && proof.source != "live" {
        return (
            false,
            format!(
                "proof source must be live when policy requires live publication, got {}",
                proof.source
            ),
        );
    }
    if proof.log_url.trim().is_empty() {
        return (false, "logUrl is required".into());
    }
    if proof.root_hash.trim().is_empty() || !is_sha_prefixed(&proof.root_hash) {
        return (false, "rootHash must be sha256-prefixed".into());
    }
    if proof.leaf_hash.trim().is_empty() || !is_sha_prefixed(&proof.leaf_hash) {
        return (false, "leafHash must be sha256-prefixed".into());
    }
    if proof.tree_size == 0 {
        return (false, "treeSize must be > 0".into());
    }
    if proof.log_index >= proof.tree_size {
        return (
            false,
            format!(
                "logIndex {} must be < treeSize {}",
                proof.log_index, proof.tree_size
            ),
        );
    }
    if policy.require_checkpoint_text && proof.checkpoint.trim().is_empty() {
        return (false, "checkpoint text is required".into());
    }
    let checkpoint_hash = format!("sha256:{}", sha256_hex_str(&proof.checkpoint));
    if checkpoint_hash != proof.checkpoint_hash {
        return (
            false,
            format!(
                "checkpointHash mismatch (expected {}, got {})",
                checkpoint_hash, proof.checkpoint_hash
            ),
        );
    }
    for (idx, digest) in proof.inclusion_path.iter().enumerate() {
        if !is_sha_prefixed(digest) {
            return (
                false,
                format!("inclusionPath[{}] is not sha256-prefixed", idx),
            );
        }
    }
    for (idx, digest) in proof.consistency_path.iter().enumerate() {
        if !is_sha_prefixed(digest) {
            return (
                false,
                format!("consistencyPath[{}] is not sha256-prefixed", idx),
            );
        }
    }
    if !proof.source.is_empty() && proof.source != "live" && proof.source != "local" {
        return (
            false,
            format!("unsupported proof source {}", proof.source),
        );
    }
    if !proof.integrated_time_utc.is_empty() && !proof.integrated_time_utc.ends_with('Z') {
        return (
            false,
            format!(
                "integratedTimeUtc must be UTC (Z suffix), got {}",
                proof.integrated_time_utc
            ),
        );
    }
    if policy.max_checkpoint_age_hours > 0 && proof.integrated_time_utc.trim().is_empty() {
        return (
            false,
            "integratedTimeUtc required by transparency freshness policy".into(),
        );
    }

    let Some(trusted) = trusted_map.get(expected_log) else {
        return (
            false,
            format!("trusted checkpoint missing for {}", expected_log),
        );
    };

    if !trusted.endpoint.is_empty() && !proof.log_url.starts_with(&trusted.endpoint) {
        return (
            false,
            format!(
                "proof logUrl {} does not match trusted endpoint {}",
                proof.log_url, trusted.endpoint
            ),
        );
    }
    if !trusted.checkpoint_hash.is_empty() && proof.checkpoint_hash != trusted.checkpoint_hash {
        return (
            false,
            format!(
                "checkpoint hash mismatch for {} (expected {}, got {})",
                expected_log, trusted.checkpoint_hash, proof.checkpoint_hash
            ),
        );
    }

    for (label, digest) in [
        ("entryDigest", proof.entry_digest.as_str()),
        ("inclusionProofHash", proof.inclusion_proof_hash.as_str()),
        (
            "consistencyProofHash",
            proof.consistency_proof_hash.as_str(),
        ),
    ] {
        if !is_sha_prefixed(digest) {
            return (
                false,
                format!("{} for {} is not sha256-prefixed", label, expected_log),
            );
        }
    }
    let inclusion_hash_expected = {
        let payload = serde_json::json!({
            "hashes": proof.inclusion_path,
            "rootHash": proof.root_hash,
        });
        format!("sha256:{}", sha256_hex_str(&canonical_json(&payload)))
    };
    if inclusion_hash_expected != proof.inclusion_proof_hash {
        return (
            false,
            format!(
                "inclusionProofHash mismatch (expected {}, got {})",
                inclusion_hash_expected, proof.inclusion_proof_hash
            ),
        );
    }
    let consistency_hash_expected = {
        let payload = serde_json::json!({
            "hashes": proof.consistency_path,
            "rootHash": proof.root_hash,
        });
        format!("sha256:{}", sha256_hex_str(&canonical_json(&payload)))
    };
    if consistency_hash_expected != proof.consistency_proof_hash {
        return (
            false,
            format!(
                "consistencyProofHash mismatch (expected {}, got {})",
                consistency_hash_expected, proof.consistency_proof_hash
            ),
        );
    }

    let signature_algorithm = if trusted.signature_algorithm.is_empty() {
        "rsa-sha256"
    } else {
        trusted.signature_algorithm.as_str()
    };
    let signature_encoding = if trusted.signature_encoding.is_empty() {
        "base64"
    } else {
        trusted.signature_encoding.as_str()
    };
    if signature_algorithm != "rsa-sha256" || signature_encoding != "base64" {
        return (
            false,
            format!(
                "unsupported trusted log signature settings for {}: {}/{}",
                expected_log, signature_algorithm, signature_encoding
            ),
        );
    }

    let signer_cert = resolve_artifact_path(&trusted.signer_certificate_path, bundle_dir);
    let chain = resolve_artifact_path(&trusted.certificate_chain_path, bundle_dir);
    let trust_roots = resolve_artifact_path(&trusted.trust_roots_path, bundle_dir);
    if !trusted.log_public_key_path.is_empty() {
        let log_public_key = resolve_artifact_path(&trusted.log_public_key_path, bundle_dir);
        if !log_public_key.exists() {
            return (
                false,
                format!(
                    "trusted log public key missing for {} at {}",
                    expected_log,
                    log_public_key.display()
                ),
            );
        }
    }
    if !signer_cert.exists() || !chain.exists() || !trust_roots.exists() {
        return (
            false,
            format!("trusted log material missing for {}", expected_log),
        );
    }
    if let Err(err) = verify_certificate_chain(&signer_cert, &chain, &trust_roots) {
        return (
            false,
            format!("transparency signer chain invalid for {}: {}", expected_log, err),
        );
    }
    let payload = transparency_signature_payload(proof);
    if let Err(err) = verify_signature_base64(&payload, &proof.signature, &signer_cert) {
        return (
            false,
            format!("transparency signature invalid for {}: {}", expected_log, err),
        );
    }
    if policy.require_entry_digest_matches_bundle {
        match declared_bundle_digest {
            Some(bundle_digest) if bundle_digest == proof.entry_digest => {}
            Some(bundle_digest) => {
                return (
                    false,
                    format!(
                        "entryDigest {} does not match declared bundle digest {}",
                        proof.entry_digest, bundle_digest
                    ),
                )
            }
            None => return (false, "declared bundle digest missing for proof binding".into()),
        }
    }
    if policy.max_checkpoint_age_hours > 0 {
        let parsed = match parse_utc(&proof.integrated_time_utc) {
            Ok(v) => v,
            Err(e) => return (false, format!("invalid integratedTimeUtc: {}", e)),
        };
        let now = Utc::now();
        if parsed > now {
            return (
                false,
                format!("integratedTimeUtc {} cannot be in the future", proof.integrated_time_utc),
            );
        }
        let age_hours = now.signed_duration_since(parsed).num_hours();
        let max_hours = freshness_policy
            .max_age_hours
            .get("transparency_proof")
            .copied()
            .unwrap_or(policy.max_checkpoint_age_hours);
        if age_hours > max_hours as i64 {
            return (
                false,
                format!(
                    "transparency proof for {} exceeded max age ({}h > {}h)",
                    expected_log, age_hours, max_hours
                ),
            );
        }
    }

    (
        true,
        format!("{} transparency proof verified", expected_log),
    )
}

fn check_badge_status(bundle: &Value, registry: &BadgeRegistry) -> (bool, String) {
    let badge_id = bundle
        .pointer("/badgeEntry/badgeId")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let status = bundle
        .pointer("/badgeEntry/status")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if badge_id.is_empty() {
        return (false, "bundle badgeId missing".into());
    }
    if status != "active" {
        return (
            false,
            format!("bundle badge status {} is not active", status),
        );
    }

    let Some(entry) = registry
        .entries
        .iter()
        .find(|entry| entry.badge_id == badge_id)
    else {
        return (false, format!("badge {} not found in registry", badge_id));
    };

    if entry.status != "active" {
        return (
            false,
            format!("registry badge {} status is {}", badge_id, entry.status),
        );
    }

    (true, format!("badge {} is active", badge_id))
}

fn topology_nodes(bundle: &Value) -> Result<Vec<TopologyNode>> {
    let nodes = bundle
        .pointer("/evidenceMap/topology/nodes")
        .ok_or_else(|| anyhow!("missing topology nodes"))?
        .clone();
    serde_json::from_value(nodes).with_context(|| "invalid topology nodes")
}

fn topology_edges(bundle: &Value) -> Result<Vec<TopologyEdge>> {
    let edges = bundle
        .pointer("/evidenceMap/topology/edges")
        .ok_or_else(|| anyhow!("missing topology edges"))?
        .clone();
    serde_json::from_value(edges).with_context(|| "invalid topology edges")
}

fn parse_csv_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>()
}

fn canonical_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
}

fn is_sha_prefixed(value: &str) -> bool {
    value.starts_with("sha256:")
        && value.len() == 71
        && value[7..].chars().all(|c| c.is_ascii_hexdigit())
}

fn hash_log_event(event: &LogEvent) -> String {
    let payload = format!(
        "{}|{}|{}|{}|{}",
        event.seq, event.prev_hash, event.payload_digest, event.event_type, event.tick_ms
    );
    format!("sha256:{}", sha256_hex_str(&payload))
}

fn transparency_signature_payload(proof: &TransparencyProof) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        proof.log_id,
        proof.entry_uuid,
        proof.log_index,
        proof.tree_size,
        proof.root_hash,
        proof.leaf_hash,
        proof.checkpoint_hash,
        proof.entry_digest,
        proof.inclusion_proof_hash,
        proof.consistency_proof_hash,
        proof.integrated_time_utc
    )
}

fn parse_utc(input: &str) -> Result<DateTime<Utc>> {
    let parsed = DateTime::parse_from_rfc3339(input)
        .with_context(|| format!("invalid RFC3339 timestamp {}", input))?;
    Ok(parsed.with_timezone(&Utc))
}

fn compute_attestation_evidence_digest(bundle: &Value, att: &AttestationEvidence) -> String {
    let profile = bundle
        .pointer("/profile")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let log_root = bundle
        .pointer("/signedOperationalLog/root")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let payload = format!(
        "profile={}|logRoot={}|softwareStateDigest={}",
        profile, log_root, att.claims.software_state_digest
    );
    format!("sha256:{}", sha256_hex_str(&payload))
}

fn resolve_policy_path(path: &Path, bundle_dir: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    if path.exists() {
        return path.to_path_buf();
    }
    bundle_dir.join(path)
}

fn temp_path(prefix: &str, extension: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("{}-{}-{}.{}", prefix, std::process::id(), nanos, extension))
}

fn verify_signature_base64(payload: &str, signature_b64: &str, signer_cert: &Path) -> Result<()> {
    let sig_bytes = base64::engine::general_purpose::STANDARD
        .decode(signature_b64)
        .with_context(|| "invalid base64 signature encoding")?;

    let payload_path = temp_path("tasc-payload", "txt");
    let sig_path = temp_path("tasc-signature", "bin");
    let pubkey_path = temp_path("tasc-pubkey", "pem");
    fs::write(&payload_path, payload.as_bytes())
        .with_context(|| format!("failed writing {}", payload_path.display()))?;
    fs::write(&sig_path, sig_bytes).with_context(|| format!("failed writing {}", sig_path.display()))?;

    let extract = Command::new("openssl")
        .args(["x509", "-in"])
        .arg(signer_cert)
        .args(["-pubkey", "-noout"])
        .output()
        .with_context(|| "failed running openssl x509 -pubkey")?;
    if !extract.status.success() {
        let _ = fs::remove_file(&payload_path);
        let _ = fs::remove_file(&sig_path);
        bail!(
            "openssl x509 -pubkey failed: {}",
            String::from_utf8_lossy(&extract.stderr)
        );
    }
    fs::write(&pubkey_path, &extract.stdout)
        .with_context(|| format!("failed writing {}", pubkey_path.display()))?;

    let verify = Command::new("openssl")
        .args(["dgst", "-sha256", "-verify"])
        .arg(&pubkey_path)
        .args(["-signature"])
        .arg(&sig_path)
        .arg(&payload_path)
        .output()
        .with_context(|| "failed running openssl dgst -verify")?;

    let _ = fs::remove_file(&payload_path);
    let _ = fs::remove_file(&sig_path);
    let _ = fs::remove_file(&pubkey_path);

    if !verify.status.success() {
        bail!(
            "openssl verify failed: {}",
            String::from_utf8_lossy(&verify.stderr)
        );
    }
    Ok(())
}

fn verify_certificate_chain(signer_cert: &Path, chain: &Path, trust_roots: &Path) -> Result<()> {
    let output = Command::new("openssl")
        .args(["verify", "-CAfile"])
        .arg(trust_roots)
        .args(["-untrusted"])
        .arg(chain)
        .arg(signer_cert)
        .output()
        .with_context(|| "failed running openssl verify")?;
    if !output.status.success() {
        bail!(
            "openssl verify failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

fn certificate_serial_hex(cert_path: &Path) -> Result<String> {
    let output = Command::new("openssl")
        .args(["x509", "-in"])
        .arg(cert_path)
        .args(["-serial", "-noout"])
        .output()
        .with_context(|| "failed running openssl x509 -serial")?;
    if !output.status.success() {
        bail!(
            "openssl x509 -serial failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    let serial = raw
        .trim()
        .split('=')
        .nth(1)
        .ok_or_else(|| anyhow!("unexpected serial output"))?;
    Ok(serial.trim().to_string())
}

fn certificate_text(cert_path: &Path) -> Result<String> {
    let output = Command::new("openssl")
        .args(["x509", "-in"])
        .arg(cert_path)
        .args(["-text", "-noout"])
        .output()
        .with_context(|| "failed running openssl x509 -text")?;
    if !output.status.success() {
        bail!(
            "openssl x509 -text failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn check_file_age_hours(path: PathBuf, max_age_hours: u64, label: &str) -> Result<()> {
    if !path.exists() {
        bail!("{} file missing: {}", label, path.display());
    }
    let modified = fs::metadata(&path)
        .with_context(|| format!("failed stat {}", path.display()))?
        .modified()
        .with_context(|| format!("failed reading mtime {}", path.display()))?;
    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or(Duration::from_secs(0));
    let max_age = Duration::from_secs(max_age_hours.saturating_mul(3600));
    if age > max_age {
        bail!(
            "{} exceeded max age: {}h > {}h for {}",
            label,
            age.as_secs() / 3600,
            max_age_hours,
            path.display()
        );
    }
    Ok(())
}

fn resolve_artifact_path(path: &str, bundle_dir: &Path) -> PathBuf {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() || candidate.exists() {
        return candidate;
    }
    let joined = bundle_dir.join(path);
    if joined.exists() {
        return joined;
    }
    for ancestor in bundle_dir.ancestors() {
        let via_ancestor = ancestor.join(path);
        if via_ancestor.exists() {
            return via_ancestor;
        }
    }
    candidate
}

fn sha256_hex_str(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

fn sha256_hex_bytes(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hex::encode(hasher.finalize())
}

fn sha256_file_prefixed(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("failed reading {}", path.display()))?;
    Ok(format!("sha256:{}", sha256_hex_bytes(&bytes)))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed reading {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("invalid JSON {}", path.display()))
}

fn read_yaml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed reading {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("invalid YAML {}", path.display()))
}
