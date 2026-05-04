/*
 * Copyright 2019, Imperva, Inc. All rights reserved.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Eclipse Public License Version 1.0, which accompanies this
 * distribution and is available at http://www.eclipse.org/legal/epl-v10.html.
 *
 * Imperva, the Imperva logo, SecureSphere, Incapsula, CounterBreach,
 * ThreatRadar, Camouflage, Attack Analytics, Prevoty and design are trademarks
 * of Imperva, Inc. and its subsidiaries. All other brand or product names are
 * trademarks or registered trademarks of their respective owners.
 */
package org.globalart.astrum.repository.cargo.internal.security;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.cargo.internal.CargoPaths;
import org.globalart.astrum.repository.cargo.internal.CargoRepositoryConfig;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.SecurityFacetSupport;
import org.globalart.astrum.repository.security.SelectorEvaluationCache;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.security.BreadActions;

import org.apache.shiro.authz.AuthorizationException;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CargoSecurityFacet
    extends SecurityFacetSupport
{
  @Autowired
  public CargoSecurityFacet(
      final CargoFormatSecurityContributor securityContributor,
      @Qualifier("simple") final VariableResolverAdapter variableResolverAdapter,
      final ContentPermissionChecker contentPermissionChecker)
  {
    super(securityContributor, variableResolverAdapter, contentPermissionChecker);
  }

  @Override
  public void ensurePermitted(final Request request, @Nullable final SelectorEvaluationCache selectorCache) {
    if (CargoRepositoryConfig.requireAuthentication(getRepository()) && CargoRepositoryConfig.isAnonymous()) {
      throw new AuthorizationException("Authentication required");
    }
    super.ensurePermitted(request, selectorCache);
  }

  @Override
  protected String action(final Request request) {
    String path = request.getPath();
    String method = request.getAction();
    if (CargoPaths.ME.equals(path)) {
      return BreadActions.READ;
    }
    if (HttpMethods.PUT.equals(method) && CargoPaths.PUBLISH.equals(path)) {
      return BreadActions.ADD;
    }
    if (path.endsWith(CargoPaths.YANK_SUFFIX) || path.endsWith(CargoPaths.UNYANK_SUFFIX)
        || path.endsWith(CargoPaths.OWNERS_SUFFIX)) {
      return HttpMethods.GET.equals(method) || HttpMethods.HEAD.equals(method) ? BreadActions.READ
          : BreadActions.EDIT;
    }
    return super.action(request);
  }
}
