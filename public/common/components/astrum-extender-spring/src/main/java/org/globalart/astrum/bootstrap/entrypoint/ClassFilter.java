
package org.globalart.astrum.bootstrap.entrypoint;

import org.springframework.lang.Nullable;

/**
 * Allows filtering classes based on package, class name regex, and type (class or interface).
 * This is used to find classes dynamically in the classpath, without using Dependency injection
 * should only be used in cases where dependency injection can't be used
 */
public class ClassFilter
{
  private String basePackage;

  @Nullable
  private String classNameRegex;

  @Nullable
  private ClassType classType;

  public String getBasePackage() {
    return basePackage;
  }

  public void setBasePackage(final String basePackage) {
    this.basePackage = basePackage;
  }

  public String getClassNameRegex() {
    return classNameRegex;
  }

  public void setClassNameRegex(final String classNameRegex) {
    this.classNameRegex = classNameRegex;
  }

  public ClassType getClassType() {
    return classType;
  }

  public void setClassType(final ClassType classType) {
    this.classType = classType;
  }

  public enum ClassType
  {
    CLASS,
    INTERFACE
  }
}
