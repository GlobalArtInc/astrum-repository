

/**
 * Preview UI string dictionary.
 *
 * Layers preview-specific strings over the base astrum-ui-plugin UIStrings.
 * Preview components should import from here rather than from
 * astrum-coreui-plugin's constants/UIStrings so they remain portable within
 * astrum-ui-plugin.
 *
 * Preview-specific keys sourced from usage:
 *   UIStrings.MALICIOUS_RISK  (MaliciousRiskStrings — all MALICIOUS_RISK.* keys)
 */
import AstrumUIStrings from '../../../constants/UIStrings';
import MaliciousRiskStrings from './pages/maliciousrisk/MaliciousRiskStrings';
import PreviewUiStrings from './pages/admin/system/PreviewUiStrings';

const PreviewUIStrings = {
  ...AstrumUIStrings,
  // Preview-specific keys referenced by components/super/:
  ...MaliciousRiskStrings,
  ...PreviewUiStrings,
};

export default PreviewUIStrings;
