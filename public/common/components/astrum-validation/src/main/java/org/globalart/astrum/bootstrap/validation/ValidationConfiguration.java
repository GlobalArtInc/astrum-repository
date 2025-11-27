
package org.globalart.astrum.bootstrap.validation;

import jakarta.validation.ConstraintValidatorFactory;
import jakarta.validation.Validation;
import jakarta.validation.Validator;
import jakarta.validation.ValidatorFactory;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.executable.ExecutableValidator;

import org.globalart.astrum.validation.internal.AlwaysTraversableResolver;
import org.globalart.astrum.validation.internal.AopAwareParanamerParameterNameProvider;

import org.hibernate.validator.messageinterpolation.ParameterMessageInterpolator;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Primary;

@Configuration
public class ValidationConfiguration
{
  /**
   * Token class used to preload validation message interpolation.
   */
  static class PreloadToken
  {
    @NotNull(message = "{org.globalart.astrum.validation.constraint.notnull}")
    String empty;
  }

  /*
   * For use by ValidationAspect
   */
  public static ExecutableValidator EXECUTABLE_VALIDATOR = null;

  @Bean
  public ValidatorFactory validatorFactory(final ConstraintValidatorFactory constraintValidatorFactory) {
    ValidatorFactory factory = Validation.byDefaultProvider()
        .configure()
        .constraintValidatorFactory(constraintValidatorFactory)
        .parameterNameProvider(new AopAwareParanamerParameterNameProvider())
        .traversableResolver(new AlwaysTraversableResolver())
        .messageInterpolator(new ParameterMessageInterpolator())
        .buildValidatorFactory();

    // FIXME: Install custom MessageInterpolator that can properly find/merge ValidationMessages.properties for bundles

    // exercise interpolator to preload elements (avoids issues later when TCCL might be different)
    factory.getValidator().validate(new PreloadToken());

    return factory;
  }

  @Primary
  @Bean
  public Validator validator(final ValidatorFactory validatorFactory) {
    Validator validator = validatorFactory.getValidator();
    EXECUTABLE_VALIDATOR = validator.forExecutables();
    return validator;
  }
}
