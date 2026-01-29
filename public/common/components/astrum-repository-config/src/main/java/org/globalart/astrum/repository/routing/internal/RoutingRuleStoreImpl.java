
package org.globalart.astrum.repository.routing.internal;

import java.util.List;
import java.util.regex.Pattern;
import java.util.regex.PatternSyntaxException;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.entity.EntityUUID;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.datastore.api.DuplicateKeyException;
import org.globalart.astrum.repository.routing.RoutingRule;
import org.globalart.astrum.repository.routing.RoutingRuleCreatedEvent;
import org.globalart.astrum.repository.routing.RoutingRuleDeletedEvent;
import org.globalart.astrum.repository.routing.RoutingRuleInvalidatedEvent;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.routing.RoutingRuleUpdatedEvent;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.transaction.Transactional;
import org.globalart.astrum.validation.constraint.NamePatternConstants;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.collect.ImmutableList;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.UUID.fromString;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * MyBatis {@link RoutingRuleStore} implementation.
 *
 * @since 3.21
 */
@Component
@Qualifier("mybatis")
public class RoutingRuleStoreImpl
    extends ConfigStoreSupport<RoutingRuleDAO>
    implements RoutingRuleStore
{
  private static final String NAME = "name";

  private static final String DESCRIPTION = "description";

  private static final String MATCHERS = "matchers";

  private static final String MODE = "mode";

  private static final String NONE = "none";

  private final EventManager eventManager;

  @Autowired
  public RoutingRuleStoreImpl(final DataSessionSupplier sessionSupplier, final EventManager eventManager) {
    super(sessionSupplier);
    this.eventManager = checkNotNull(eventManager);
  }

  @Override
  public RoutingRule newRoutingRule() {
    return new RoutingRuleData();
  }

  @Transactional
  @Override
  public List<RoutingRule> list() {
    return ImmutableList.copyOf(dao().browse());
  }

  @Transactional
  @Override
  public RoutingRule create(final RoutingRule rule) {
    try {
      dao().create((RoutingRuleData) validate(rule));
      eventManager.post(new RoutingRuleCreatedEvent(rule));
      return rule;
    }
    catch (DuplicateKeyException e) {
      throw new ValidationErrorsException("name", "A rule with the same name already exists. Name must be unique.");
    }
  }

  @Transactional
  @Override
  public RoutingRule getById(final String id) {
    return dao().read(new EntityUUID(fromString(id))).orElse(null);
  }

  @Transactional
  @Override
  public RoutingRule getByName(final String name) {
    return dao().readByName(name).orElse(null);
  }

  @Override
  public void update(final RoutingRule rule) {
    doUpdate((RoutingRuleData) validate(rule));
    eventManager.post(new RoutingRuleUpdatedEvent(rule));
    postEvent(rule);
  }

  @Transactional
  protected void doUpdate(final RoutingRuleData rule) {
    try {
      dao().update(rule);
    }
    catch (DuplicateKeyException e) {
      throw new ValidationErrorsException("name", "A rule with the same name already exists. Name must be unique.");
    }
  }

  @Override
  public void delete(final RoutingRule rule) {
    doDelete(rule.name());
    eventManager.post(new RoutingRuleDeletedEvent(rule));
    postEvent(rule);
  }

  @Transactional
  protected void doDelete(final String name) {
    dao().deleteByName(name);
  }

  private void postEvent(final RoutingRule rule) {
    eventManager.post(new RoutingRuleInvalidatedEvent(rule.id()));
  }

  @VisibleForTesting
  static RoutingRule validate(final RoutingRule rule) { // NOSONAR
    ValidationErrorsException exception = new ValidationErrorsException();

    if (Strings2.isBlank(rule.name())) {
      exception.withError(NAME, "A non-empty value must be specified");
    }
    else if (!rule.name().matches(NamePatternConstants.REGEX)) {
      exception.withError(NAME,
          "Only letters, digits, underscores(_), hyphens(-), and dots(.) are allowed and may not start with underscore or dot.");
    }
    else if (rule.name().equalsIgnoreCase(NONE)) {
      exception.withError(NAME, "Rule must not be named None");
    }

    if (rule.description() == null) {
      exception.withError(DESCRIPTION, "A non-null value must be specified");
    }

    if (rule.mode() == null) {
      exception.withError(MODE, "A non-empty value must be specified");
    }

    if (rule.matchers() == null || rule.matchers().isEmpty()) {
      exception.withError(MATCHERS, "At least one rule must be specified");
    }
    else {
      int index = 0;
      for (String regex : rule.matchers()) {
        if (Strings2.isBlank(regex)) {
          exception.withError(MATCHERS + "[" + index + "]", "Empty matchers are not allowed");
        }
        else {
          try {
            Pattern.compile(regex);
          }
          catch (PatternSyntaxException e) { // NOSONAR
            exception.withError(MATCHERS + "[" + index + "]", "Invalid regex: " + e.getMessage());
          }
        }
        index++;
      }
    }

    if (!exception.getValidationErrors().isEmpty()) {
      throw exception;
    }

    return rule;
  }
}
