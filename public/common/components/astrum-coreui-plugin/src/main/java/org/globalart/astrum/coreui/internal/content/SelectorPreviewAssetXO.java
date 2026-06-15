
package org.globalart.astrum.coreui.internal.content;

/**
 * Response DTO for the content-selector preview endpoint. Deliberately narrow — exposes only the fields the UI
 * needs so callers with selector-management permission but no browse rights on individual repositories cannot
 * learn asset metadata beyond a name and repository label. See ASTRUM-52615.
 */
public record SelectorPreviewAssetXO(String name, String repositoryName)
{
}
