
package org.globalart.astrum.extdirect.model;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.Set;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.ConstraintViolationException;
import jakarta.validation.ElementKind;
import jakarta.validation.Path.Node;

import com.google.common.base.Joiner;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Ext.Direct validation response.
 *
 * @since 3.0
 */
public class ValidationResponse
    extends Response<Object>
{
  private List<String> messages;

  private Map<String, String> errors;

  public ValidationResponse(final ConstraintViolationException cause) {
    super(false, new ArrayList<>());
    // noinspection ThrowableResultOfMethodCallIgnored
    checkNotNull(cause);
    Set<ConstraintViolation<?>> violations = cause.getConstraintViolations();
    if (violations != null && !violations.isEmpty()) {
      for (ConstraintViolation<?> violation : violations) {
        List<String> entries = new ArrayList<>();
        // iterate path to get the full path
        Iterator<Node> it = violation.getPropertyPath().iterator();
        while (it.hasNext()) {
          Node node = it.next();
          if (ElementKind.PROPERTY == node.getKind() || (ElementKind.PARAMETER == node.getKind() && !it.hasNext())) {
            if (node.getKey() != null) {
              entries.add(node.getKey().toString());
            }
            entries.add(node.getName());
          }
        }
        if (entries.isEmpty()) {
          if (messages == null) {
            messages = new ArrayList<>();
          }
          messages.add(violation.getMessage());
        }
        else {
          if (errors == null) {
            errors = new HashMap<>();
          }
          errors.put(Joiner.on('.').join(entries), violation.getMessage());
        }
      }
    }
    else if (cause.getMessage() != null) {
      messages = new ArrayList<>();
      messages.add(cause.getMessage());
    }
  }

  public List<String> getMessages() {
    return messages;
  }
}
