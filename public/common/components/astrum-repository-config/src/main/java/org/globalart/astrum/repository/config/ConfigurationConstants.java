
package org.globalart.astrum.repository.config;

/**
 * Repository configuration constants.
 *
 * @since 3.22
 */
public interface ConfigurationConstants
{
  String STORAGE = "storage";

  String DATA_STORE_NAME = "dataStoreName";

  String BLOB_STORE_NAME = "blobStoreName";

  String WRITE_POLICY = "writePolicy";

  /**
   * Docker specific constant indicating whether the 'latest' tag should be allowed to be updated with the ALLOW_ONCE
   * write policy.
   */
  String LATEST_POLICY = "latestPolicy";

  String WRITE_POLICY_DEFAULT = "ALLOW";

  String STRICT_CONTENT_TYPE_VALIDATION = "strictContentTypeValidation";

  String GROUP_WRITE_MEMBER = "groupWriteMember";

  String COMPONENT = "component";

  String PROPRIETARY_COMPONENTS = "proprietaryComponents";

  /**
   * Attribute key for the per-repository firewall configuration sub-map (mode etc.).
   * Mirrors the literal already used by the typed REST API converters and downstream readers;
   * exposed here so non-REST-API callers (e.g. {@code BaseRepositoryManager}) do not have to
   * import {@code rest.api.model.FirewallAttributes}.
   */
  String FIREWALL = "firewall";
}
