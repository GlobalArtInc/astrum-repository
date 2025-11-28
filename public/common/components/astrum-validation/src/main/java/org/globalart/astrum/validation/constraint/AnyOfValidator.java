
package org.globalart.astrum.validation.constraint;

import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.Stream;
import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.validation.ConstraintValidatorSupport;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AnyOfValidator
    extends ConstraintValidatorSupport<AnyOf, String>
{
  Set<String> values;

  String paramName;

  String message;

  public AnyOfValidator() {
  }

  @Override
  public void initialize(final AnyOf annotation) {
    Class<? extends Enum<?>> enumClass = annotation.enumClass();

    paramName = annotation.parameterName();
    message = annotation.message();
    values = Stream.of(enumClass.getEnumConstants())
        .map(Enum::name)
        .map(String::toLowerCase)
        .collect(Collectors.toSet());
  }

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    if (value == null) {
      return true;
    }

    boolean isValid = values.contains(value.toLowerCase());

    if (!isValid) {
      context.disableDefaultConstraintViolation();
      context.buildConstraintViolationWithTemplate(getEscapeHelper().stripJavaEl(buildMessage()))
          .addConstraintViolation();
    }

    return isValid;
  }

  private String buildMessage() {
    return String.format(message, paramName, String.join(",", values));
  }
}
