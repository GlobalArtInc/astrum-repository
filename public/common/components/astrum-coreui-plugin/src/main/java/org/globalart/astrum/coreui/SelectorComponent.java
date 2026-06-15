
package org.globalart.astrum.coreui;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintViolationException;
import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.groups.Default;

import org.apache.shiro.SecurityUtils;
import org.apache.shiro.authz.annotation.RequiresPermissions;

import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;
import org.globalart.astrum.repository.security.RepositoryContentSelectorPrivilegeDescriptor;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.privilege.Privilege;
import org.globalart.astrum.selector.SelectorConfiguration;
import org.globalart.astrum.selector.SelectorConfigurationStore;
import org.globalart.astrum.selector.SelectorFactory;
import org.globalart.astrum.selector.SelectorManager;
import org.globalart.astrum.validation.ConstraintViolationFactory;
import org.globalart.astrum.validation.Validate;
import org.globalart.astrum.validation.group.Create;
import org.globalart.astrum.validation.group.Update;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;

import jakarta.validation.constraints.NotEmpty;
import java.util.Comparator;
import java.util.Collections;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Selector {@link DirectComponent}.
 */
@Component
@DirectAction(action = "coreui_Selector")
public class SelectorComponent
    extends DirectComponentSupport
{
  private static final String EXPRESSION_KEY = "expression";

  private final SelectorManager selectorManager;

  private final ConstraintViolationFactory constraintViolationFactory;

  private final SelectorFactory selectorFactory;

  private final SecuritySystem securitySystem;

  private final SelectorConfigurationStore store;

  @Autowired
  public SelectorComponent(
      final SelectorManager selectorManager,
      final ConstraintViolationFactory constraintViolationFactory,
      final SelectorFactory selectorFactory,
      final SecuritySystem securitySystem,
      final SelectorConfigurationStore store)
  {
    this.selectorManager = checkNotNull(selectorManager);
    this.constraintViolationFactory = checkNotNull(constraintViolationFactory);
    this.selectorFactory = checkNotNull(selectorFactory);
    this.securitySystem = checkNotNull(securitySystem);
    this.store = checkNotNull(store);
  }

  /**
   * @return a list of selectors
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:selectors:read")
  public List<SelectorXO> read() {
    Set<Privilege> privileges = securitySystem.listPrivileges();
    return store.browse()
        .stream()
        .map(config -> asSelector(config, privileges))
        .collect(Collectors.toList()); // NOSONAR
  }

  /**
   * Creates a selector.
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:selectors:create")
  @Validate(groups = {Create.class, Default.class})
  public SelectorXO create(@NotNull @Valid final SelectorXO selectorXO) {
    selectorFactory.validateSelector(selectorXO.getType(), selectorXO.getExpression());

    SelectorConfiguration configuration = selectorManager.newSelectorConfiguration(
        selectorXO.getName(), selectorXO.getType(), selectorXO.getDescription(),
        Collections.singletonMap(EXPRESSION_KEY, selectorXO.getExpression()));
    selectorManager.create(configuration);
    return asSelector(configuration, securitySystem.listPrivileges());
  }

  /**
   * Updates a selector.
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:selectors:update")
  @Validate(groups = {Update.class, Default.class})
  public SelectorXO update(@NotNull @Valid final SelectorXO selectorXO) {
    selectorFactory.validateSelector(selectorXO.getType(), selectorXO.getExpression());
    SelectorConfiguration config = selectorManager.readByName(selectorXO.getName());
    config.setDescription(selectorXO.getDescription());
    config.setAttributes(Collections.singletonMap(EXPRESSION_KEY, selectorXO.getExpression()));
    selectorManager.update(config);
    return selectorXO;
  }

  /**
   * Deletes a selector.
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:selectors:delete")
  @Validate
  public void remove(@NotEmpty final String name) {
    try {
      selectorManager.delete(selectorManager.readByName(name));
    }
    catch (IllegalStateException e) {
      throw new ConstraintViolationException(e.getMessage(),
          Collections.singleton(constraintViolationFactory.createViolation("*", e.getMessage())));
    }
  }

  /**
   * Retrieve a list of available selector references.
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:selectors:read")
  public List<ReferenceXO> readReferences() {
    return selectorManager.browse()
        .stream()
        .map(config -> new ReferenceXO(config.getName(), config.getName()))
        .sorted(Comparator.comparing(ReferenceXO::getName, String.CASE_INSENSITIVE_ORDER)
            .thenComparing(ReferenceXO::getName))
        .collect(Collectors.toList()); // NOSONAR
  }

  private SelectorXO asSelector(final SelectorConfiguration configuration, final Set<Privilege> privilegeSet) {
    List<String> privileges = getPrivilegesUsingSelector(configuration, privilegeSet);

    SelectorXO selectorXO = new SelectorXO();
    selectorXO.setId(configuration.getName());
    selectorXO.setName(configuration.getName());
    selectorXO.setType(configuration.getType());
    selectorXO.setDescription(configuration.getDescription());
    selectorXO.setExpression(configuration.getAttributes().get(EXPRESSION_KEY));
    selectorXO.setUsedBy(canReadPrivileges() ? privileges : Collections.emptyList());
    selectorXO.setUsedByCount(privileges.size());

    return selectorXO;
  }

  private List<String> getPrivilegesUsingSelector(
      final SelectorConfiguration selectorConfiguration,
      final Set<Privilege> privileges)
  {
    return privileges.stream()
        .filter(privilege -> RepositoryContentSelectorPrivilegeDescriptor.TYPE.equals(privilege.getType()))
        .filter(privilege -> selectorConfiguration.getName()
            .equals(privilege.getProperties().get(RepositoryContentSelectorPrivilegeDescriptor.P_CONTENT_SELECTOR)))
        .map(Privilege::getName)
        .collect(Collectors.toList()); // NOSONAR
  }

  private static boolean canReadPrivileges() {
    return SecurityUtils.getSubject().isPermitted("astrum:privileges:read");
  }
}
