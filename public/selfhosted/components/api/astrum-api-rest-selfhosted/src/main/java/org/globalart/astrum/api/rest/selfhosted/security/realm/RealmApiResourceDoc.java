
package org.globalart.astrum.api.rest.selfhosted.security.realm;

import java.util.List;

import org.globalart.astrum.api.rest.selfhosted.security.realm.model.RealmApiXO;

import io.swagger.v3.oas.annotations.tags.Tag;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.Parameter;

/**
 * @since 3.20
 */
@Tag(name = "Security management: realms")
public interface RealmApiResourceDoc
{
  @Operation(summary = "List the available realms")
  List<RealmApiXO> getRealms();

  @Operation(summary = "List the active realm IDs in order")
  List<String> getActiveRealms();

  @Operation(summary = "Set the active security realms in the order they should be used")
  void setActiveRealms(@Parameter(description = "The realm IDs") List<String> realmIds);
}
