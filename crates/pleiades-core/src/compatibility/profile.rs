#![forbid(unsafe_code)]

use core::fmt;

use pleiades_ayanamsa::AyanamsaDescriptor;
use pleiades_houses::{house_system_code_aliases, HouseSystemCodeAlias, HouseSystemDescriptor};

use super::report::{format_canonical_name_summary, format_string_summary};
use super::validation::{
    validate_house_code_aliases, validate_profile_text_section, CompatibilityProfileValidationError,
};

/// A release-scoped compatibility profile.
#[derive(Clone, Copy, Debug)]
pub struct CompatibilityProfile {
    /// Stable profile identifier.
    pub profile_id: &'static str,
    /// Human-readable summary of the current release posture.
    pub summary: &'static str,
    /// Scope note describing the long-term house-system target.
    pub target_house_scope: &'static [&'static str],
    /// Scope note describing the long-term ayanamsa target.
    pub target_ayanamsa_scope: &'static [&'static str],
    /// Built-in house systems shipped in this release line.
    pub house_systems: &'static [HouseSystemDescriptor],
    /// House systems that belong to the published baseline milestone.
    pub baseline_house_systems: &'static [HouseSystemDescriptor],
    /// Release-specific house-system additions beyond the baseline milestone.
    pub release_house_systems: &'static [HouseSystemDescriptor],
    /// Built-in ayanamsas shipped in this release line.
    pub ayanamsas: &'static [AyanamsaDescriptor],
    /// Built-in ayanamsas that belong to the published baseline milestone.
    pub baseline_ayanamsas: &'static [AyanamsaDescriptor],
    /// Release-specific ayanamsa additions beyond the baseline milestone.
    pub release_ayanamsas: &'static [AyanamsaDescriptor],
    /// Explicitly documented release-specific notes beyond the baseline milestone.
    pub release_notes: &'static [&'static str],
    /// Validation reference points that are intentionally surfaced separately
    /// from unresolved compatibility gaps.
    pub validation_reference_points: &'static [&'static str],
    /// Labels that are intentionally surfaced as custom-definition territory
    /// instead of unresolved compatibility gaps.
    pub custom_definition_labels: &'static [&'static str],
    /// Explicitly documented compatibility caveats and follow-up notes.
    pub known_gaps: &'static [&'static str],
}

/// Typed summary of the built-in Swiss-Ephemeris house-code alias inventory.
#[derive(Clone, Copy, Debug)]
pub struct HouseCodeAliasInventorySummary {
    aliases: &'static [HouseSystemCodeAlias],
}

impl HouseCodeAliasInventorySummary {
    /// Creates a summary for the provided house-code alias inventory.
    pub const fn new(aliases: &'static [HouseSystemCodeAlias]) -> Self {
        Self { aliases }
    }

    /// Returns the number of short-form aliases in the inventory.
    pub fn count(&self) -> usize {
        self.aliases.len()
    }

    /// Returns the compact rendered short-code mapping.
    pub fn summary_line(&self) -> String {
        self.aliases
            .iter()
            .map(HouseSystemCodeAlias::summary_line)
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Returns the compact rendered short-code mapping after validation.
    pub fn validated_summary_line(&self) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Validates the inventory against the release-profile house-code rules.
    pub fn validate(&self) -> Result<usize, CompatibilityProfileValidationError> {
        validate_house_code_aliases(self.aliases)
    }
}

impl fmt::Display for HouseCodeAliasInventorySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl CompatibilityProfile {
    /// Returns a short release note string.
    pub const fn release_note(&self) -> &'static str {
        self.summary
    }

    /// Returns the release note after validating the profile's release-facing metadata.
    pub fn validated_release_note(
        &self,
    ) -> Result<&'static str, CompatibilityProfileValidationError> {
        self.validate()?;
        Ok(self.summary)
    }

    /// Returns the long-term target house-system scope as a compact summary.
    pub fn target_house_scope_summary_line(&self) -> String {
        self.target_house_scope.join("; ")
    }

    /// Returns the long-term target house-system scope after validating the profile.
    pub fn validated_target_house_scope_summary_line(
        &self,
    ) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        Ok(self.target_house_scope_summary_line())
    }

    /// Returns the long-term target ayanamsa scope as a compact summary.
    pub fn target_ayanamsa_scope_summary_line(&self) -> String {
        self.target_ayanamsa_scope.join("; ")
    }

    /// Returns the long-term target ayanamsa scope after validating the profile.
    pub fn validated_target_ayanamsa_scope_summary_line(
        &self,
    ) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        Ok(self.target_ayanamsa_scope_summary_line())
    }

    /// Returns a typed summary of the Swiss-Ephemeris house-code alias inventory.
    pub fn house_code_alias_inventory_summary(&self) -> HouseCodeAliasInventorySummary {
        HouseCodeAliasInventorySummary::new(house_system_code_aliases())
    }

    /// Returns the number of Swiss Ephemeris house-table code aliases.
    pub fn house_code_alias_count(&self) -> usize {
        self.house_code_alias_inventory_summary().count()
    }

    /// Returns the Swiss-Ephemeris house-code alias table as compact mappings.
    ///
    /// # Examples
    ///
    /// ```
    /// use pleiades_core::current_compatibility_profile;
    ///
    /// let summary = current_compatibility_profile().house_code_aliases_summary_line();
    /// assert!(summary.contains("P -> Placidus"));
    /// ```
    pub fn house_code_aliases_summary_line(&self) -> String {
        self.house_code_alias_inventory_summary().summary_line()
    }

    /// Returns the house-code alias inventory after validating the profile.
    pub fn validated_house_code_aliases_summary_line(
        &self,
    ) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        self.house_code_alias_inventory_summary()
            .validated_summary_line()
    }

    /// Returns the release-specific house-system canonical names as a compact summary.
    pub fn release_house_system_canonical_names_summary_line(&self) -> String {
        format_canonical_name_summary(&self.release_house_system_canonical_names())
    }

    /// Returns the release-specific house-system canonical names after validating the profile.
    pub fn validated_release_house_system_canonical_names_summary_line(
        &self,
    ) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        let names = self.release_house_system_canonical_names();
        validate_profile_text_section("release-house-system-canonical-name", names.as_slice())?;
        Ok(format_canonical_name_summary(&names))
    }

    /// Returns the release-specific ayanamsa canonical names as a compact summary.
    pub fn release_ayanamsa_canonical_names_summary_line(&self) -> String {
        format_canonical_name_summary(&self.release_ayanamsa_canonical_names())
    }

    /// Returns the release-specific ayanamsa canonical names after validating the profile.
    pub fn validated_release_ayanamsa_canonical_names_summary_line(
        &self,
    ) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        let names = self.release_ayanamsa_canonical_names();
        validate_profile_text_section("release-ayanamsa-canonical-name", names.as_slice())?;
        Ok(format_canonical_name_summary(&names))
    }

    /// Returns the representative ayanamsa provenance payload surfaced in the compatibility profile.
    pub fn ayanamsa_provenance_summary_line(&self) -> String {
        super::ayanamsa_provenance_summary_text()
    }

    /// Returns the representative ayanamsa provenance payload after validating the profile.
    pub fn validated_ayanamsa_provenance_summary_line(
        &self,
    ) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        Ok(super::ayanamsa_provenance_summary_text())
    }

    /// Returns the custom-definition ayanamsa labels surfaced in the compatibility profile.
    pub fn custom_definition_ayanamsa_labels(&self) -> Vec<&'static str> {
        pleiades_ayanamsa::metadata_coverage().custom_definition_only
    }

    /// Returns the custom-definition ayanamsa labels as a compact human-readable line.
    pub fn custom_definition_ayanamsa_labels_summary_line(&self) -> String {
        format_canonical_name_summary(&self.custom_definition_ayanamsa_labels())
    }

    /// Returns the custom-definition ayanamsa labels after validating the profile.
    pub fn validated_custom_definition_ayanamsa_labels_summary_line(
        &self,
    ) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        let labels = self.custom_definition_ayanamsa_labels();
        validate_profile_text_section("custom-definition-ayanamsa-label", labels.as_slice())?;
        Ok(format_canonical_name_summary(&labels))
    }

    /// Returns the number of latitude-sensitive house-system descriptors.
    pub fn constrained_house_system_count(&self) -> usize {
        self.house_systems
            .iter()
            .filter(|entry| entry.latitude_sensitive)
            .count()
    }

    /// Returns the number of ayanamsa descriptors that still lack sidereal metadata.
    pub fn ayanamsa_descriptor_only_count(&self) -> usize {
        self.ayanamsas
            .iter()
            .filter(|entry| !entry.has_sidereal_metadata())
            .count()
    }

    pub(super) fn ayanamsa_alias_bearing_entry_count(&self) -> usize {
        self.ayanamsas
            .iter()
            .filter(|entry| !entry.aliases.is_empty())
            .count()
    }

    /// Returns a compact inventory line for the current compatibility catalog.
    pub fn catalog_inventory_summary_line(&self) -> String {
        fn alias_count<T>(entries: &[T], aliases: impl Fn(&T) -> &'static [&'static str]) -> usize {
            entries.iter().map(|entry| aliases(entry).len()).sum()
        }

        let house_alias_count = alias_count(self.house_systems, |entry| entry.aliases);
        let house_code_alias_count = self.house_code_alias_count();
        let ayanamsa_alias_count = alias_count(self.ayanamsas, |entry| entry.aliases);
        let custom_definition_ayanamsa_labels = self.custom_definition_ayanamsa_labels();
        let ayanamsa_metadata_gap_count = pleiades_ayanamsa::metadata_coverage()
            .without_sidereal_metadata
            .len();
        let ayanamsa_alias_bearing_entry_count = self.ayanamsa_alias_bearing_entry_count();
        let ayanamsa_provenance = self.ayanamsa_provenance_summary_line();
        let constrained_house_system_count = self.constrained_house_system_count();
        let unconstrained_house_system_count = self
            .house_systems
            .len()
            .saturating_sub(constrained_house_system_count);
        let ayanamsa_descriptor_only_count = self.ayanamsa_descriptor_only_count();
        let metadata_bearing_ayanamsa_count = self
            .ayanamsas
            .len()
            .saturating_sub(ayanamsa_descriptor_only_count);

        let mut text = String::from("Compatibility catalog inventory: ");
        text.push_str("house systems=");
        text.push_str(&self.house_systems.len().to_string());
        text.push_str(" (");
        text.push_str(&self.baseline_house_systems.len().to_string());
        text.push_str(" baseline, ");
        text.push_str(&self.release_house_systems.len().to_string());
        text.push_str(" release-specific, ");
        text.push_str(&house_alias_count.to_string());
        text.push_str(" aliases); house formula families=");
        text.push_str(&self.house_formula_families_summary_line());
        text.push_str("; house latitude-sensitive constraints=");
        text.push_str(&self.latitude_sensitive_house_constraints_summary_line());
        text.push_str("; house-code aliases=");
        text.push_str(&house_code_alias_count.to_string());
        text.push_str("; ayanamsas=");
        text.push_str(&self.ayanamsas.len().to_string());
        text.push_str(" (");
        text.push_str(&self.baseline_ayanamsas.len().to_string());
        text.push_str(" baseline, ");
        text.push_str(&self.release_ayanamsas.len().to_string());
        text.push_str(" release-specific, ");
        text.push_str(&ayanamsa_alias_count.to_string());
        text.push_str(" aliases); custom-definition labels=");
        text.push_str(&self.custom_definition_labels.len().to_string());
        text.push_str("; custom-definition ayanamsa labels=");
        text.push_str(&custom_definition_ayanamsa_labels.len().to_string());
        text.push_str(" (");
        text.push_str(&custom_definition_ayanamsa_labels.join(", "));
        text.push_str("); ayanamsa metadata gaps=");
        text.push_str(&ayanamsa_metadata_gap_count.to_string());
        text.push_str("; ayanamsa alias-bearing entries=");
        text.push_str(&ayanamsa_alias_bearing_entry_count.to_string());
        text.push_str("; catalog posture=house systems=");
        text.push_str(&self.house_systems.len().to_string());
        text.push_str(" (");
        text.push_str(&constrained_house_system_count.to_string());
        text.push_str(" constrained, ");
        text.push_str(&unconstrained_house_system_count.to_string());
        text.push_str(" unconstrained); ayanamsas=");
        text.push_str(&self.ayanamsas.len().to_string());
        text.push_str(" (");
        text.push_str(&ayanamsa_descriptor_only_count.to_string());
        text.push_str(" descriptor-only, ");
        text.push_str(&metadata_bearing_ayanamsa_count.to_string());
        text.push_str(" metadata-bearing); custom-only labels=");
        text.push_str(&self.custom_definition_labels.len().to_string());
        text.push_str("; custom-only ayanamsa labels=");
        text.push_str(&custom_definition_ayanamsa_labels.len().to_string());
        text.push_str("; ayanamsa provenance=");
        text.push_str(&ayanamsa_provenance);
        text.push_str("; known gaps=");
        text.push_str(&self.known_gaps.len().to_string());
        text.push_str("; claim audit: baseline catalogs are the published guarantees; release-specific entries are shipped additions; custom-definition labels remain custom-definition territory; descriptor-only ayanamsa entries remain catalog descriptors; constrained house systems stay explicitly flagged; known gaps stay documented");
        text
    }

    /// Returns the catalog inventory after validating the profile.
    pub fn validated_catalog_inventory_summary_line(
        &self,
    ) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        Ok(self.catalog_inventory_summary_line())
    }

    /// Returns the compact known-gaps line for the current compatibility profile.
    pub fn known_gaps_summary_line(&self) -> String {
        format_canonical_name_summary(self.known_gaps)
    }

    /// Returns the known-gaps line after validating the profile.
    pub fn validated_known_gaps_summary_line(
        &self,
    ) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        Ok(self.known_gaps_summary_line())
    }

    /// Returns the unsupported-advanced-modes posture line surfaced in release reporting.
    pub const fn unsupported_modes_summary_line(&self) -> &'static str {
        super::CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT
    }

    /// Returns the compact catalog-posture line for the current compatibility profile.
    pub fn catalog_posture_summary_line(&self) -> String {
        let constrained_house_system_count = self.constrained_house_system_count();
        let unconstrained_house_system_count = self
            .house_systems
            .len()
            .saturating_sub(constrained_house_system_count);
        let ayanamsa_descriptor_only_count = self.ayanamsa_descriptor_only_count();
        let metadata_bearing_ayanamsa_count = self
            .ayanamsas
            .len()
            .saturating_sub(ayanamsa_descriptor_only_count);

        format!(
            "Catalog posture: house systems={} descriptors ({} constrained, {} unconstrained); ayanamsas={} descriptors ({} metadata-bearing, {} descriptor-only); ayanamsa alias-bearing entries={}; ayanamsa metadata gaps={}; custom-definition labels={}; custom-definition ayanamsa labels={}; known gaps={}",
            self.house_systems.len(),
            constrained_house_system_count,
            unconstrained_house_system_count,
            self.ayanamsas.len(),
            metadata_bearing_ayanamsa_count,
            ayanamsa_descriptor_only_count,
            self.ayanamsa_alias_bearing_entry_count(),
            ayanamsa_descriptor_only_count,
            self.custom_definition_labels.len(),
            self.custom_definition_ayanamsa_labels().len(),
            self.known_gaps_summary_line()
        )
    }

    /// Returns the catalog-posture line after validating the profile.
    pub fn validated_catalog_posture_summary_line(
        &self,
    ) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        Ok(self.catalog_posture_summary_line())
    }

    /// Returns the built-in house systems that are latitude-sensitive.
    pub fn latitude_sensitive_house_systems(&self) -> Vec<&'static str> {
        self.house_systems
            .iter()
            .filter(|entry| entry.latitude_sensitive)
            .map(|entry| entry.canonical_name)
            .collect()
    }

    /// Returns the latitude-sensitive house-system coverage as a compact human-readable line.
    pub fn latitude_sensitive_house_systems_summary_line(&self) -> String {
        format_canonical_name_summary(&self.latitude_sensitive_house_systems())
    }

    /// Returns the latitude-sensitive house-system coverage after validating the profile.
    pub fn validated_latitude_sensitive_house_systems_summary_line(
        &self,
    ) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        Ok(self.latitude_sensitive_house_systems_summary_line())
    }

    /// Returns the release-facing latitude-sensitive constraint notes.
    pub fn latitude_sensitive_house_constraints(&self) -> Vec<String> {
        self.house_systems
            .iter()
            .filter(|entry| entry.latitude_sensitive)
            .map(|entry| format!("{} [{}]", entry.canonical_name, entry.notes))
            .collect()
    }

    /// Returns the latitude-sensitive house constraints as a compact human-readable line.
    pub fn latitude_sensitive_house_constraints_summary_line(&self) -> String {
        format_string_summary(&self.latitude_sensitive_house_constraints())
    }

    /// Returns the latitude-sensitive house constraints after validating the profile.
    pub fn validated_latitude_sensitive_house_constraints_summary_line(
        &self,
    ) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        Ok(self.latitude_sensitive_house_constraints_summary_line())
    }

    /// Returns the release-facing failure-mode notes for latitude-sensitive house systems.
    pub fn latitude_sensitive_house_failure_modes(&self) -> Vec<String> {
        self.house_systems
            .iter()
            .filter(|entry| entry.latitude_sensitive)
            .map(|entry| format!("{} [{}]", entry.canonical_name, entry.notes))
            .collect()
    }

    /// Returns the latitude-sensitive house failure modes as a compact human-readable line.
    pub fn latitude_sensitive_house_failure_modes_summary_line(&self) -> String {
        format_string_summary(&self.latitude_sensitive_house_failure_modes())
    }

    /// Returns the unique house formula families represented in the profile,
    /// sorted by their release-facing labels.
    pub fn house_formula_family_names(&self) -> Vec<String> {
        use std::collections::BTreeSet;
        self.house_systems
            .iter()
            .map(|entry| entry.formula_family().to_string())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Returns the house-formula-family coverage as a compact human-readable line.
    pub fn house_formula_families_summary_line(&self) -> String {
        let names = self.house_formula_family_names();
        match names.as_slice() {
            [] => "0 (none)".to_string(),
            [single] => format!("1 ({single})"),
            _ => format!("{} ({})", names.len(), names.join(", ")),
        }
    }

    /// Returns the house-formula-family coverage after validating the profile.
    pub fn validated_house_formula_families_summary_line(
        &self,
    ) -> Result<String, CompatibilityProfileValidationError> {
        self.validate()?;
        Ok(self.house_formula_families_summary_line())
    }

    /// Returns the canonical names for the built-in house-system baseline.
    pub fn baseline_house_system_canonical_names(&self) -> Vec<&'static str> {
        Self::canonical_names(self.baseline_house_systems, |entry| entry.canonical_name)
    }

    /// Returns the canonical names for the release-specific house-system additions.
    pub fn release_house_system_canonical_names(&self) -> Vec<&'static str> {
        Self::canonical_names(self.release_house_systems, |entry| entry.canonical_name)
    }

    /// Returns the canonical names for the built-in ayanamsa baseline.
    pub fn baseline_ayanamsa_canonical_names(&self) -> Vec<&'static str> {
        Self::canonical_names(self.baseline_ayanamsas, |entry| entry.canonical_name)
    }

    /// Returns the canonical names for the release-specific ayanamsa additions.
    pub fn release_ayanamsa_canonical_names(&self) -> Vec<&'static str> {
        Self::canonical_names(self.release_ayanamsas, |entry| entry.canonical_name)
    }

    fn canonical_names<T>(
        entries: &[T],
        canonical_name: impl Fn(&T) -> &'static str,
    ) -> Vec<&'static str> {
        entries.iter().map(canonical_name).collect()
    }
}
