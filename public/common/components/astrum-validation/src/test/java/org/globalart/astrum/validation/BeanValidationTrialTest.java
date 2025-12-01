
package org.globalart.astrum.validation;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.Valid;
import jakarta.validation.Validation;
import jakarta.validation.ValidatorFactory;
import jakarta.validation.constraints.NotNull;

import org.junit.Before;
import org.junit.Test;

import java.util.Map;
import java.util.Set;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.hasSize;

public class BeanValidationTrialTest

{

  private ValidatorFactory factory;

  @Before
  public void setUp() {
    factory = Validation.buildDefaultValidatorFactory();
  }

  private static class Parent
  {
    @Valid
    private Child child;

    public Parent(Child child) {
      this.child = child;
    }
  }

  private static class Child
  {
    @Valid
    private GrandChild grandChild;

    public Child(GrandChild grandChild) {
      this.grandChild = grandChild;
    }
  }

  private static class GrandChild
  {
    @NotNull
    private String name;

    public GrandChild(String name) {
      this.name = name;
    }
  }

  @Test
  public void shouldValidateHierarchy() {
    Parent parent = new Parent(new Child(new GrandChild(null)));
    Set<ConstraintViolation<Parent>> violations = factory.getValidator().validate(parent);
    assertThat(violations, hasSize(1));
    ConstraintViolation<Parent> violation = violations.iterator().next();
    assertThat(violation.getPropertyPath().toString(), equalTo("child.grandChild.name"));
    assertThat(violation.getMessage(), equalTo("must not be null"));
  }

  private static class WithMap
  {
    @Valid
    private Map<String, Object> contents;

    public WithMap(Map<String, Object> contents) {
      this.contents = contents;
    }
  }

  @Test
  public void shouldValidateWithMap() {
    Parent parent = new Parent(new Child(new GrandChild(null)));
    WithMap withMap = new WithMap(Map.of("foo", parent));
    Set<ConstraintViolation<WithMap>> violations = factory.getValidator().validate(withMap);
    assertThat(violations, hasSize(1));
    ConstraintViolation<WithMap> violation = violations.iterator().next();
    assertThat(violation.getPropertyPath().toString(), equalTo("contents[foo].child.grandChild.name"));
    assertThat(violation.getMessage(), equalTo("must not be null"));
  }
}
