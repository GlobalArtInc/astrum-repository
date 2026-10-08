
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
 * Test to prevent usage of PostgreSQL extendSchema with actual SQL statements.
 *
 * Related to ASTRUM-49154: extendSchema with PostgreSQL should not contain DDL statements
 * (CREATE TABLE, ALTER TABLE, CREATE INDEX, etc.) as they run on every startup and can
 * cause lock contention in HA environments.
 *
 * Pattern: Use versioned migrations instead of extendSchema for PostgreSQL DDL.
 *
 * H2 extendSchema is still allowed (uses IF NOT EXISTS which is safe for H2).
 *
 * @since 3.87
 */
public class ExtendSchemaUsageTest
{
  private static final String PROJECT_ROOT = findProjectRoot();

  @Test
  public void testNoPostgresqlExtendSchemaWithSql() throws Exception {
    List<String> violations = new ArrayList<>();

    try (Stream<Path> paths = Files.walk(Paths.get(PROJECT_ROOT))) {
      paths.filter(Files::isRegularFile)
          .filter(p -> p.toString().endsWith(".xml"))
          .filter(p -> p.toString().contains("src/main/resources"))
          .forEach(xmlFile -> {
            try {
              String content = Files.readString(xmlFile);
              checkPostgresqlExtendSchema(xmlFile, content, violations);
            }
            catch (IOException e) {
              // Skip files that can't be read
            }
          });
    }

    assertThat(
        "Found PostgreSQL extendSchema sections with SQL statements.\n\n" +
            "ASTRUM-49154: extendSchema should not contain DDL for PostgreSQL as it runs on every startup.\n" +
            "Solution: Move DDL to:\n" +
            "  1. createSchema (for new installations)\n" +
            "  2. Versioned migration (for existing databases)\n" +
            "  3. Leave PostgreSQL extendSchema empty with comment\n\n" +
            "H2 extendSchema is still allowed (IF NOT EXISTS is safe for H2).\n\n" +
            "Violations:\n" + String.join("\n", violations),
        violations,
        empty());
  }

  private void checkPostgresqlExtendSchema(Path xmlFile, String content, List<String> violations) {
    // Look for PostgreSQL extendSchema sections
    int pos = 0;
    while ((pos = content.indexOf("extendSchema", pos)) != -1) {
      // Find the databaseId attribute
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

      // Check if this is PostgreSQL
      if (!insertTag.contains("databaseId=\"PostgreSQL\"") && !insertTag.contains("databaseId='PostgreSQL'")) {
        pos++;
        continue;
      }

      // Find the closing tag
      int closeTag = content.indexOf("</insert>", endTag);
      if (closeTag == -1) {
        pos++;
        continue;
      }

      String sectionContent = content.substring(endTag + 1, closeTag).trim();

      // Check if it contains SQL statements (not just comments)
      if (containsSqlStatements(sectionContent)) {
        String relativePath = Paths.get(PROJECT_ROOT).relativize(xmlFile).toString();
        violations.add(String.format("  - %s\n    Contains: %s",
            relativePath,
            getSqlStatementSummary(sectionContent)));
      }

      pos = closeTag;
    }
  }

  private boolean containsSqlStatements(String content) {
    // Remove XML comments
    String withoutComments = content.replaceAll("<!--.*?-->", "");

    // Remove SQL line comments
    withoutComments = withoutComments.replaceAll("--[^\n]*", "");

    // Remove whitespace
    String trimmed = withoutComments.trim();

    // If nothing left, it's just comments
    if (trimmed.isEmpty()) {
      return false;
    }

    // Check for SQL DDL keywords
    String upperContent = trimmed.toUpperCase();
    return upperContent.contains("CREATE ") ||
        upperContent.contains("ALTER ") ||
        upperContent.contains("DROP ") ||
        upperContent.contains("INSERT ") ||
        upperContent.contains("UPDATE ") ||
        upperContent.contains("DELETE ");
  }

  private String getSqlStatementSummary(String content) {
    String summary = content.replaceAll("<!--.*?-->", "")
        .replaceAll("--[^\n]*", "")
        .trim();

    if (summary.length() > 100) {
      summary = summary.substring(0, 100) + "...";
    }

    return summary.replaceAll("\\s+", " ");
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
