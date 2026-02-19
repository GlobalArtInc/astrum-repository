
package org.globalart.astrum.repository.rest.api.model;

import java.util.Collection;

import jakarta.validation.constraints.NotEmpty;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

/**
 * REST API model used to describe a group repository.
 *
 * @since 3.20
 */
public class GroupAttributes
{
  @Schema(description = "Member repositories' names")
  @NotEmpty
  protected final Collection<String> memberNames;

  @JsonCreator
  public GroupAttributes(@JsonProperty("memberNames") final Collection<String> memberNames) {
    this.memberNames = memberNames;
  }

  public Collection<String> getMemberNames() {
    return memberNames;
  }
}
