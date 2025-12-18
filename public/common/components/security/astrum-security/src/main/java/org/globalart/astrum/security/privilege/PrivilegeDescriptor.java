
package org.globalart.astrum.security.privilege;

import java.util.List;

import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.privilege.rest.ApiPrivilege;
import org.globalart.astrum.security.privilege.rest.ApiPrivilegeRequest;

import org.apache.shiro.authz.Permission;

/**
 * Privilege descriptor.
 */
public interface PrivilegeDescriptor<T extends ApiPrivilege, Y extends ApiPrivilegeRequest>
{
  String getType();

  String getName();

  Permission createPermission(CPrivilege privilege);

  List<FormField> getFormFields();

  T createApiPrivilegeImpl(Privilege privilege);

  void validate(Y apiPrivilege);
}
