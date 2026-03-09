
package org.globalart.astrum.script;

import java.util.Objects;
import jakarta.validation.constraints.NotEmpty;

/**
 * Script result exchange object.
 *
 * @since 3.0
 */
public class ScriptResultXO
{

  @NotEmpty
  private String name;

  private String result;

  public ScriptResultXO() {
  }

  public ScriptResultXO(String name, String result) {
    this.name = name;
    this.result = result;
  }

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  public String getResult() {
    return result;
  }

  public void setResult(String result) {
    this.result = result;
  }

  @Override
  public boolean equals(Object o) {
    if (this == o)
      return true;
    if (o == null || getClass() != o.getClass())
      return false;
    ScriptResultXO that = (ScriptResultXO) o;
    return Objects.equals(name, that.name) && Objects.equals(result, that.result);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, result);
  }

  @Override
  public String toString() {
    return "ScriptResultXO{" +
        "name='" + name + '\'' +
        ", result='" + result + '\'' +
        '}';
  }
}
