package org.globalart.astrum.repository.terraform.internal.hosted;

import java.util.Map;
import java.util.Set;

import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.upload.UploadDefinition;
import org.globalart.astrum.repository.upload.UploadFieldDefinition;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;
import static org.mockito.Mockito.mock;

public class TerraformUploadHandlerTest
{
  @Test
  public void tellsModulesFromProvidersByTheirCoordinates() {
    assertThat(TerraformUploadHandler.uploadType(Map.of("namespace", "acme", "name", "vpc", "system", "aws")))
        .isEqualTo("module");
    assertThat(TerraformUploadHandler.uploadType(Map.of("namespace", "acme", "type", "demo"))).isEqualTo("provider");
    assertThat(TerraformUploadHandler.uploadType(Map.of("uploadType", "Provider", "name", "x"))).isEqualTo("provider");
    assertThatThrownBy(() -> TerraformUploadHandler.uploadType(Map.of("namespace", "acme")))
        .isInstanceOf(ValidationErrorsException.class);
    assertThatThrownBy(() -> TerraformUploadHandler.uploadType(Map.of("uploadType", "chart")))
        .isInstanceOf(ValidationErrorsException.class);
  }

  @Test
  public void readsPluginProtocolVersions() {
    assertThat(TerraformUploadHandler.protocols("5.0, 6")).containsExactly("5.0", "6.0");
    assertThat(TerraformUploadHandler.protocols(" ")).isNull();
    assertThatThrownBy(() -> TerraformUploadHandler.protocols("six")).isInstanceOf(ValidationErrorsException.class);
  }

  @Test
  public void definesFieldsForModulesAndProviders() {
    TerraformUploadHandler handler = new TerraformUploadHandler(mock(ContentPermissionChecker.class),
        mock(VariableResolverAdapter.class), Set.of());

    UploadDefinition definition = handler.getDefinition();

    assertThat(definition.getFormat()).isEqualTo("terraform");
    assertThat(definition.isMultipleUpload()).isTrue();
    assertThat(definition.getComponentFields()).extracting(UploadFieldDefinition::getName)
        .containsExactly("uploadType", "namespace", "version", "name", "system", "type", "protocols");
    assertThat(definition.getComponentFields()).filteredOn(field -> !field.isOptional())
        .extracting(UploadFieldDefinition::getName).containsExactly("namespace", "version");
  }
}
