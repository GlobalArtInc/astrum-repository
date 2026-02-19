
package org.globalart.astrum.repository.rest.api.model;

import java.util.Collection;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

/**
 * REST API model which describes cleanup policies for a repository.
 *
 * @since 3.20
 */
public class CleanupPolicyAttributes
{
  @Schema(description = "Components that match any of the applied policies will be deleted")
  protected Collection<String> policyNames;

  @JsonCreator
  public CleanupPolicyAttributes(@JsonProperty("policyNames") final Collection<String> policyNames) {
    this.policyNames = policyNames;
  }

  public Collection<String> getPolicyNames() {
    return policyNames;
  }
}
