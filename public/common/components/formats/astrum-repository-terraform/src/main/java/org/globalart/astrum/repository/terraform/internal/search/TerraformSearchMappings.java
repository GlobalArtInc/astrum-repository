package org.globalart.astrum.repository.terraform.internal.search;

import java.util.List;

import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.SearchMappings;
import org.globalart.astrum.repository.terraform.TerraformFormat;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_1;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_2;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_3;
import static org.globalart.astrum.repository.rest.sql.SearchField.FORMAT_FIELD_4;

/**
 * Search by the parts of a module address ({@code namespace/name/system}) or a provider address
 * ({@code namespace/type}); {@code terraform.provider} is the system of a module and the type of a provider.
 */
@Component
@Qualifier(TerraformFormat.NAME)
public class TerraformSearchMappings
    implements SearchMappings
{
  private static final List<SearchMapping> MAPPINGS = List.of(
      new SearchMapping("terraform.namespace", "assets.attributes.terraform.namespace", "Terraform namespace",
          FORMAT_FIELD_1, false),
      new SearchMapping("terraform.name", "assets.attributes.terraform.name", "Terraform module or provider name",
          FORMAT_FIELD_2, false),
      new SearchMapping("terraform.provider", "assets.attributes.terraform.provider",
          "Terraform module system or provider type", FORMAT_FIELD_3, false),
      new SearchMapping("terraform.kind", "assets.attributes.terraform.kind", "Terraform kind (module or provider)",
          FORMAT_FIELD_4, false));

  @Override
  public Iterable<SearchMapping> get() {
    return MAPPINGS;
  }
}
