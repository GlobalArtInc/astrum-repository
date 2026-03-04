
package org.globalart.astrum.selector;

import java.io.File;
import java.net.URL;
import java.nio.file.Path;
import java.nio.file.Paths;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.apache.commons.jexl3.JexlException;
import org.junit.Test;

import static com.google.common.collect.Streams.stream;
import static org.globalart.astrum.selector.CselValidator.validateCselExpression;

public class CselValidatorTest
{
  public static final String BASEDIR = new File(System.getProperty("basedir", "")).getAbsolutePath();

  private JexlEngine engine = new JexlEngine();

  private ObjectMapper mapper = new ObjectMapper();

  @Test
  public void parsesAllValidContentSelectors() throws Exception {
    URL jsonFile = this.getClass().getResource("/validJexlContentSelectors.json");
    JsonNode contentSelectors = mapper.readTree(jsonFile);
    stream(contentSelectors).map(JsonNode::asText).forEach(this::validateExpression);
  }

  @Test(expected = JexlException.Parsing.class)
  public void failsToParseInvalidContentSelectors() throws Exception {
    validateExpression("invalid content selector");
  }

  @Test(expected = JexlException.class)
  public void failsToParseCoordinateContentSelectors() throws Exception {
    validateExpression("coordinate.groupId == \"com.globalart\"");
  }

  @Test(expected = JexlException.class)
  public void failsToValidateInvalidContentSelectors() throws Exception {
    validateExpression("a.b.c = false");
  }

  @Test(expected = JexlException.class)
  public void failsToValidateEmbeddedSingleQuoteInStrings() throws Exception {
    validateExpression("format == \"'\"");
  }

  @Test(expected = JexlException.class)
  public void failsToValidateEmbeddedDoubleQuoteInStrings() throws Exception {
    validateExpression("format == '\"'");
  }

  @Test(expected = JexlException.class)
  public void failsToValidateInvalidRegex() throws Exception {
    validateExpression("path =~ '*foo*'");
  }

  public static File resolveBaseFile(final String path) {
    return resolveBasePath(path).toFile();
  }

  public static Path resolveBasePath(final String path) {
    return Paths.get(BASEDIR, path);
  }

  private void validateExpression(final String expression) {
    validateCselExpression(engine.parseExpression(expression));
  }
}
