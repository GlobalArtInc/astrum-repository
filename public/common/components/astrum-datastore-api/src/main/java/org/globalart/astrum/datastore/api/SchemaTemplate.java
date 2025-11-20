
package org.globalart.astrum.datastore.api;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/**
 * Marks a {@link DataAccess} template whose schema contains a placeholder variable.
 *
 * To use a template extend the type and prefix the simple name. The lower-case form
 * of that prefix will be used to replace the placeholder variable in the template:
 *
 * <pre>
 * &#64;SchemaTemplate("format")
 * public interface AssetDAO
 * {
 *   // DAO whose schema is a template containing ${format}
 * }
 *
 * public interface MavenAssetDAO
 *     extends AssetDAO
 * {
 *   // uses AssetDAO schema with ${format} replaced by maven
 * }
 * </pre>
 *
 * @since 3.20
 */
@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.TYPE)
public @interface SchemaTemplate
{
  /**
   * The name of the placeholder variable in the schema.
   */
  String value();
}
