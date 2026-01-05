
package org.globalart.astrum.security.privilege.rest;

import io.swagger.v3.jaxrs2.Reader;
import io.swagger.v3.oas.models.OpenAPI;
import io.swagger.v3.oas.models.media.ArraySchema;
import io.swagger.v3.oas.models.media.ComposedSchema;
import io.swagger.v3.oas.models.media.Schema;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.globalart.astrum.swagger.internal.PrivilegeApiSwaggerContributor;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.greaterThanOrEqualTo;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.instanceOf;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;

/**
 * Integration test verifying that {@link PrivilegeApiSwaggerContributor} correctly patches
 * the OpenAPI spec generated from the real {@link PrivilegeApiResourceV1}.
 *
 * <p>
 * Scans the actual JAX-RS resource (no Spring context needed — the Reader only reads
 * annotations) and asserts that the GET privilege operations' 200 responses expose a
 * {@code oneOf} union rather than the abstract {@code ApiPrivilege} base schema.
 */
public class PrivilegeApiSwaggerIT
{
  private OpenAPI openApi;

  @BeforeEach
  public void setUp() {
    // Scan the real resource the same way SwaggerModel does at runtime
    Reader reader = new Reader(new OpenAPI());
    reader.read(PrivilegeApiResourceV1.class);
    openApi = reader.getOpenAPI();

    // Apply the contributor under test
    new PrivilegeApiSwaggerContributor().contribute(openApi);
  }

  @Test
  public void listPrivileges_200Response_isArrayOfOneOf() {
    Schema<?> schema = get200Schema("/v1/security/privileges");

    assertThat("list schema should be an array", schema, instanceOf(ArraySchema.class));

    Schema<?> items = ((ArraySchema) schema).getItems();
    assertThat("array items should be a oneOf union", items, instanceOf(ComposedSchema.class));
    assertThat("oneOf should contain at least the two astrum-security subtypes",
        ((ComposedSchema) items).getOneOf(), hasSize(greaterThanOrEqualTo(2)));
  }

  @Test
  public void getPrivilege_200Response_isOneOf() {
    Schema<?> schema = get200Schema("/v1/security/privileges/{privilegeName}");

    assertThat("single-get schema should be a oneOf union", schema, instanceOf(ComposedSchema.class));
    assertThat("oneOf should contain at least the two astrum-security subtypes",
        ((ComposedSchema) schema).getOneOf(), hasSize(greaterThanOrEqualTo(2)));
  }

  @Test
  public void listPrivileges_oneOf_containsApplicationAndWildcard() {
    // This IT lives in astrum-security whose test classpath includes only astrum-security classes.
    // ApiPrivilegeRepositoryAdmin/View/ContentSelector (astrum-repository-services) and
    // ApiPrivilegeScript (astrum-script-plugin) are NOT on the test classpath here, so the
    // contributor's Class.forName silently skips them — only 2 of 6 subtypes are verifiable.
    // The full 6-subtype union is exercised at runtime where all bundles are on the TCCL.
    Schema<?> items = ((ArraySchema) get200Schema("/v1/security/privileges")).getItems();
    ComposedSchema oneOf = (ComposedSchema) items;

    boolean hasApplication = oneOf.getOneOf()
        .stream()
        .anyMatch(s -> s.get$ref() != null && s.get$ref().contains("ApiPrivilegeApplication"));
    boolean hasWildcard = oneOf.getOneOf()
        .stream()
        .anyMatch(s -> s.get$ref() != null && s.get$ref().contains("ApiPrivilegeWildcard"));

    assertThat("oneOf should reference ApiPrivilegeApplication", hasApplication, is(true));
    assertThat("oneOf should reference ApiPrivilegeWildcard", hasWildcard, is(true));
  }

  @Test
  public void listPrivileges_oneOf_hasTypeDiscriminator() {
    Schema<?> items = ((ArraySchema) get200Schema("/v1/security/privileges")).getItems();
    ComposedSchema oneOf = (ComposedSchema) items;

    assertThat(oneOf.getDiscriminator(), is(notNullValue()));
    assertThat(oneOf.getDiscriminator().getPropertyName(), is("type"));
  }

  @Test
  public void listPrivileges_subtypeSchemas_registeredInComponents() {
    assertThat(openApi.getComponents(), is(notNullValue()));
    assertThat("ApiPrivilegeApplication should be in components",
        openApi.getComponents().getSchemas().containsKey("ApiPrivilegeApplication"), is(true));
    assertThat("ApiPrivilegeWildcard should be in components",
        openApi.getComponents().getSchemas().containsKey("ApiPrivilegeWildcard"), is(true));
  }

  // ---- helper ----

  private Schema<?> get200Schema(final String path) {
    assertThat("path " + path + " not found in spec", openApi.getPaths().get(path), notNullValue());
    return openApi.getPaths()
        .get(path)
        .getGet()
        .getResponses()
        .get("200")
        .getContent()
        .get("application/json")
        .getSchema();
  }
}
