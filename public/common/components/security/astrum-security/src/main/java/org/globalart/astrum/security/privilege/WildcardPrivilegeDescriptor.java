
package org.globalart.astrum.security.privilege;

import java.util.List;

import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.common.i18n.I18N;
import org.globalart.astrum.common.i18n.MessageBundle;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.StringTextFormField;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.security.authz.WildcardPermission2;
import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.CPrivilegeBuilder;
import org.globalart.astrum.security.privilege.rest.ApiPrivilegeWildcard;
import org.globalart.astrum.security.privilege.rest.ApiPrivilegeWildcardRequest;

import com.google.common.collect.ImmutableList;
import org.apache.shiro.authz.Permission;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Wildcard {@link PrivilegeDescriptor}.
 *
 * @see WildcardPermission2
 * @since 3.0
 */
@Component
@Qualifier(WildcardPrivilegeDescriptor.TYPE)
public class WildcardPrivilegeDescriptor
    extends PrivilegeDescriptorSupport<ApiPrivilegeWildcard, ApiPrivilegeWildcardRequest>
{
  public static final String TYPE = "wildcard";

  public static final String P_PATTERN = "pattern";

  private interface Messages
      extends MessageBundle
  {
    @DefaultMessage("Wildcard")
    String name();

    @DefaultMessage("Privilege String")
    String pattern();

    @DefaultMessage("The internal segment matching algorithm uses Apache Shiro wildcard permissions")
    String patternHelp();
  }

  private static final Messages messages = I18N.create(Messages.class);

  private final List<FormField> formFields;

  public WildcardPrivilegeDescriptor() {
    super(TYPE);
    this.formFields = ImmutableList.of(
        new StringTextFormField(
            P_PATTERN,
            messages.pattern(),
            messages.patternHelp(),
            FormField.MANDATORY));
  }

  @Override
  public Permission createPermission(final CPrivilege privilege) {
    checkNotNull(privilege);
    String pattern = readProperty(privilege, P_PATTERN);
    return new WildcardPermission2(pattern);
  }

  @Override
  public List<FormField> getFormFields() {
    return formFields;
  }

  @Override
  public String getName() {
    return messages.name();
  }

  //
  // Helpers
  //

  public static String id(final String pattern) {
    return pattern;
  }

  public static CPrivilege privilege(final String pattern) {
    return new CPrivilegeBuilder()
        .type(TYPE)
        .id(id(pattern))
        .property(P_PATTERN, pattern)
        .create();
  }

  @Override
  public ApiPrivilegeWildcard createApiPrivilegeImpl(final Privilege privilege) {
    return new ApiPrivilegeWildcard(privilege);
  }

  @Override
  public void validate(final ApiPrivilegeWildcardRequest apiPrivilege) {
    if (apiPrivilege.getPattern() == null || apiPrivilege.getPattern().isBlank()) {
      throw new WebApplicationMessageException(Status.BAD_REQUEST,
          "\"Wildcard privilege pattern must not be blank.\"",
          MediaType.APPLICATION_JSON);
    }
  }
}
