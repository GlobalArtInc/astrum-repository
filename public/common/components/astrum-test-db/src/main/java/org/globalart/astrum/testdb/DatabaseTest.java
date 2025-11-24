
package org.globalart.astrum.testdb;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

import org.junit.jupiter.api.TestTemplate;
import org.junit.jupiter.api.extension.ExtendWith;

/**
 * Used to indicate that a test should be run with databases. See {@link DatabaseExtension} for a description of usage.
 */
@Target({ElementType.ANNOTATION_TYPE, ElementType.METHOD, ElementType.TYPE})
@Retention(RetentionPolicy.RUNTIME)
@TestTemplate
@ExtendWith(DatabaseExtension.class)
public @interface DatabaseTest
{
  /**
   * Can be used to disable creating a test run of the annotated method using the H2 database. Both this and the
   * related {@link DataSessionConfiguration#h2()} must be true for the test to run with H2.
   */
  boolean h2() default true;

  /**
   * Can be used to enable creating a test run of the annotated method using the PostgreSQL database. Both this and
   * the related {@link DataSessionConfiguration#postgresql()} must be true for the test to run with PostgreSQL.
   */
  boolean postgresql() default false;
}
