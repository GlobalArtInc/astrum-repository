
package org.globalart.astrum.repository.core;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Stream;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.empty;

/**
 * Test to prevent usage of ALTER TABLE and CREATE INDEX in MyBatis createSchema sections.
 *
 * Related to ASTRUM-49154: createSchema should not contain ALTER TABLE or CREATE INDEX
 * statements as they can cause lock contention on PostgreSQL startup in HA environments.
 *
 * Pattern: Use versioned migrations for all ALTER TABLE and CREATE INDEX operations.
 *
 * createSchema should ONLY contain:
 * - CREATE TABLE statements (with IF NOT EXISTS)
 * - PRIMARY KEY constraints (inline in CREATE TABLE)
 * - FOREIGN KEY constraints (inline in CREATE TABLE)
 * - UNIQUE constraints (inline in CREATE TABLE)
 *
 * createSchema should NOT contain:
 * - ALTER TABLE statements (use migrations instead)
 * - CREATE INDEX statements (use migrations instead)
 *
 * @since 3.87
 */
public class CreateSchemaUsageTest
{
  private static final String PROJECT_ROOT = findProjectRoot();

  @Test
  public void testNoAlterTableOrCreateIndexInCreateSchema() throws Exception {
    List<String> violations = new ArrayList<>();

    try (Stream<Path> paths = Files.walk(Paths.get(PROJECT_ROOT))) {
      paths.filter(Files::isRegularFile)
          .filter(p -> p.toString().endsWith("DAO.xml"))
          .filter(p -> p.toString().contains("src/main/resources"))
          .forEach(xmlFile -> {
            try {
              String content = Files.readString(xmlFile);
              checkCreateSchema(xmlFile, content, violations);
            }
            catch (IOException e) {
              // Skip files that can't be read
            }
          });
    }

    assertThat(
        "Found createSchema sections with ALTER TABLE or CREATE INDEX statements.\n\n" +
            "ASTRUM-49154: createSchema should not contain ALTER TABLE or CREATE INDEX.\n" +
            "These statements cause PostgreSQL lock contention on startup in HA environments.\n\n" +
            "Solution:\n" +
            "  1. Remove ALTER TABLE and CREATE INDEX from createSchema\n" +
            "  2. Create versioned migration (DatabaseMigrationStep) instead\n" +
            "  3. Migration should check tableExists() and indexExists() before creating\n\n" +
            "Allowed in createSchema:\n" +
            "  - CREATE TABLE IF NOT EXISTS\n" +
            "  - Inline constraints (PRIMARY KEY, FOREIGN KEY, UNIQUE in CREATE TABLE)\n\n" +
            "Not allowed in createSchema:\n" +
            "  - ALTER TABLE (use migration)\n" +
            "  - CREATE INDEX (use migration)\n\n" +
            "Violations:\n" + String.join("\n", violations),
        violations,
        empty());
  }

  private void checkCreateSchema(Path xmlFile, String content, List<String> violations) {
    // Look for createSchema sections
    int pos = 0;
    while ((pos = content.indexOf("createSchema", pos)) != -1) {
      // Find the opening tag
      int startTag = content.lastIndexOf("<insert", pos);
      if (startTag == -1) {
        pos++;
        continue;
      }

      int endTag = content.indexOf(">", startTag);
      if (endTag == -1) {
        pos++;
        continue;
      }

      String insertTag = content.substring(startTag, endTag + 1);

      // Must have id="createSchema"
      if (!insertTag.contains("id=\"createSchema\"") && !insertTag.contains("id='createSchema'")) {
        pos++;
        continue;
      }

      // Find the closing tag
      int closeTag = content.indexOf("</insert>", endTag);
      if (closeTag == -1) {
        pos++;
        continue;
      }

      String sectionContent = content.substring(endTag + 1, closeTag);

      // Check for ALTER TABLE or CREATE INDEX
      List<String> problems = checkForProhibitedStatements(sectionContent);
      if (!problems.isEmpty()) {
        String relativePath = Paths.get(PROJECT_ROOT).relativize(xmlFile).toString();
        String databaseId = extractDatabaseId(insertTag);
        violations.add(String.format("  - %s%s\n    %s",
            relativePath,
            databaseId.isEmpty() ? "" : " (" + databaseId + ")",
            String.join("\n    ", problems)));
      }

      pos = closeTag;
    }
  }

  private List<String> checkForProhibitedStatements(String content) {
    List<String> problems = new ArrayList<>();

    // Remove XML comments
    String withoutXmlComments = content.replaceAll("<!--.*?-->", "");

    // Remove SQL line comments
    String withoutComments = withoutXmlComments.replaceAll("--[^\n]*", "");

    // Check for ALTER TABLE
    if (containsStatement(withoutComments, "ALTER TABLE")) {
      problems.add("Contains ALTER TABLE (should use migration instead)");
    }

    // Check for CREATE INDEX
    if (containsStatement(withoutComments, "CREATE INDEX") ||
        containsStatement(withoutComments, "CREATE UNIQUE INDEX")) {
      problems.add("Contains CREATE INDEX (should use migration instead)");
    }

    return problems;
  }

  private boolean containsStatement(String content, String statement) {
    // Remove whitespace for easier matching
    String normalized = content.replaceAll("\\s+", " ").toUpperCase();
    return normalized.contains(statement);
  }

  private String extractDatabaseId(String insertTag) {
    // Extract databaseId="xxx" or databaseId='xxx'
    int dbIdStart = insertTag.indexOf("databaseId");
    if (dbIdStart == -1) {
      return "";
    }

    int quoteStart = insertTag.indexOf("\"", dbIdStart);
    if (quoteStart == -1) {
      quoteStart = insertTag.indexOf("'", dbIdStart);
    }
    if (quoteStart == -1) {
      return "";
    }

    int quoteEnd = insertTag.indexOf("\"", quoteStart + 1);
    if (quoteEnd == -1) {
      quoteEnd = insertTag.indexOf("'", quoteStart + 1);
    }
    if (quoteEnd == -1) {
      return "";
    }

    return insertTag.substring(quoteStart + 1, quoteEnd);
  }

  private static String findProjectRoot() {
    File current = new File(".").getAbsoluteFile();
    while (current != null) {
      if (new File(current, "pom.xml").exists() &&
          new File(current, "components").exists()) {
        return current.getAbsolutePath();
      }
      current = current.getParentFile();
    }
    throw new IllegalStateException("Could not find project root");
  }
}
