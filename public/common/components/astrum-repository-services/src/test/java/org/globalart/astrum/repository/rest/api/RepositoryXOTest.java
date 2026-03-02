
package org.globalart.astrum.repository.rest.api;

import java.util.Arrays;
import java.util.Collection;
import java.util.Map;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;

import org.junit.Rule;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnit;
import org.mockito.junit.MockitoRule;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@RunWith(Parameterized.class)
public class RepositoryXOTest
{
  @Rule
  public MockitoRule mockitoRule = MockitoJUnit.rule().silent();

  @Mock
  private Repository repository;

  private final String name;

  private final Format format;

  private final String expectedFormat;

  private final Type type;

  private final String expectedType;

  private final String url;

  private final boolean online;

  private final Map<String, Object> attributes;

  private final Map<String, Map<String, Object>> expectedAttributes;

  public RepositoryXOTest(
      final String name,
      final Format format,
      final String expectedFormat,
      final Type type,
      final String expectedType,
      final String url,
      final boolean online,
      final Map<String, Object> attributes,
      final Map<String, Map<String, Object>> expectedAttributes)
  {
    this.name = name;
    this.format = format;
    this.expectedFormat = expectedFormat;
    this.type = type;
    this.expectedType = expectedType;
    this.url = url;
    this.online = online;
    this.attributes = attributes;
    this.expectedAttributes = expectedAttributes;
  }

  @Parameterized.Parameters
  public static Collection<Object[]> data() {
    return Arrays.asList(new Object[][]{
        {
            "x", format("npm"), "npm", new ProxyType(), "proxy", "u", true, Map.of("remoteUrl", "url"),
            Map.of("proxy", Map.of("remoteUrl", "url"))
        },
        {"y", format("maven"), "maven", new HostedType(), "hosted", "u", false, Map.of("remoteUrl", "foo"), Map.of()},
        {"z", format("nuget"), "nuget", new GroupType(), "group", "u", true, Map.of("remoteUrl", "foo"), Map.of()}
    });
  }

  @Test
  public void testConvertRepositoryToRepositoryXO() {
    when(repository.getName()).thenReturn(name);
    when(repository.getFormat()).thenReturn(format);
    when(repository.getType()).thenReturn(type);
    when(repository.getUrl()).thenReturn(url);

    Configuration mockConfiguration = configuration(type.getValue(), online, attributes);
    when(repository.getConfiguration()).thenReturn(mockConfiguration);

    RepositoryXO repositoryXO = RepositoryXO.fromRepository(repository);

    assertThat(repositoryXO.getName(), is(name));
    assertThat(repositoryXO.getFormat(), is(expectedFormat));
    assertThat(repositoryXO.getType(), is(expectedType));
    assertThat(repositoryXO.getUrl(), is(url));
    assertThat(repositoryXO.isOnline(), is(online));
    assertThat(repositoryXO.getAttributes(), is(expectedAttributes));
  }

  private static Format format(final String value) {
    Format format = mock(Format.class);
    when(format.getValue()).thenReturn(value);
    return format;
  }

  private Configuration configuration(final String type, final boolean online, final Map<String, Object> value) {
    Configuration configuration = mock(Configuration.class);
    when(configuration.getAttributes()).thenReturn(Map.of(type, value));
    when(configuration.attributes(type)).thenReturn(new NestedAttributesMap(type, value));
    when(configuration.isOnline()).thenReturn(online);
    return configuration;
  }
}
