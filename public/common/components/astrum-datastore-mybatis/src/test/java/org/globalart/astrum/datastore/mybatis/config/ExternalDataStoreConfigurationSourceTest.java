package org.globalart.astrum.datastore.mybatis.config;

import java.io.File;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.datastore.api.DataStoreConfiguration;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class ExternalDataStoreConfigurationSourceTest
{
  @Rule
  public TemporaryFolder temporaryFolder = new TemporaryFolder();

  @Test
  public void environmentSourceIsDisabledWithoutJdbcUrl() {
    EnvironmentDataStoreConfigurationSource source = new EnvironmentDataStoreConfigurationSource(Map.of());
    assertThat(source.isEnabled(), is(false));
    assertThat(source.browseStoreNames().iterator().hasNext(), is(false));
  }

  @Test
  public void environmentSourceReadsTheAstrumStoreVariables() {
    EnvironmentDataStoreConfigurationSource source = new EnvironmentDataStoreConfigurationSource(Map.of(
        "ASTRUM_DATASTORE_ASTRUM_JDBCURL", "jdbc:postgresql://db:5432/astrum",
        "ASTRUM_DATASTORE_ASTRUM_USERNAME", "astrum",
        "ASTRUM_DATASTORE_ASTRUM_PASSWORD", "secret",
        "ASTRUM_DATASTORE_ASTRUM_MAXIMUMPOOLSIZE", "50"));
    assertThat(source.isEnabled(), is(true));
    assertThat(List.copyOf((java.util.Collection<String>) source.browseStoreNames()), is(List.of("astrum")));

    DataStoreConfiguration configuration = source.load("astrum");
    assertThat(configuration.getSource(), is("environment"));
    assertThat(configuration.getType(), is("jdbc"));
    assertThat(configuration.getAttributes(), is(Map.of(
        "jdbcUrl", "jdbc:postgresql://db:5432/astrum",
        "username", "astrum",
        "password", "secret",
        "maximumPoolSize", "50")));
  }

  @Test
  public void fileSourceReadsTheStorePropertiesFile() throws Exception {
    File work = temporaryFolder.newFolder("work");
    Path fabric = work.toPath().resolve("etc/fabric");
    Files.createDirectories(fabric);
    ApplicationDirectories directories = mock(ApplicationDirectories.class);
    when(directories.getWorkDirectory("etc/fabric")).thenReturn(fabric.toFile());
    FileDataStoreConfigurationSource source = new FileDataStoreConfigurationSource(directories);
    assertThat(source.isEnabled(), is(false));

    Files.writeString(fabric.resolve("astrum-store.properties"),
        "jdbcUrl=jdbc:postgresql://db:5432/astrum\nusername=astrum\npassword=secret\nschema=repo\n");
    assertThat(source.isEnabled(), is(true));
    assertThat(source.load("astrum").getAttributes(), is(Map.of(
        "jdbcUrl", "jdbc:postgresql://db:5432/astrum",
        "username", "astrum",
        "password", "secret",
        "schema", "repo")));
    assertThat(source.load("astrum").getSource(), is("file"));
  }
}
