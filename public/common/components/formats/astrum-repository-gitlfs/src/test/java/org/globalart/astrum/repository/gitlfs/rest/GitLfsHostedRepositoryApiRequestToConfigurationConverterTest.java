package org.globalart.astrum.repository.gitlfs.rest;

import java.util.List;
import java.util.Set;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationStore;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.gitlfs.internal.hosted.GitLfsHostedRecipe;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;

import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class GitLfsHostedRepositoryApiRequestToConfigurationConverterTest
{
  @Test
  void convertsARequestToAHostedConfiguration() {
    ConfigurationStore configurationStore = mock(ConfigurationStore.class);
    when(configurationStore.newConfiguration()).thenReturn(new ConfigurationData());
    GitLfsHostedRepositoryApiRequestToConfigurationConverter underTest =
        new GitLfsHostedRepositoryApiRequestToConfigurationConverter();
    underTest.setConfigurationStore(configurationStore);

    Configuration configuration = underTest.convert(new GitLfsHostedRepositoryApiRequest("lfs", true,
        new HostedStorageAttributes("default", false, "ALLOW_ONCE"),
        new CleanupPolicyAttributes(List.of("stale-objects")), null));

    assertThat(configuration.getRepositoryName(), is("lfs"));
    assertThat(configuration.getRecipeName(), is(GitLfsHostedRecipe.NAME));
    assertThat(configuration.isOnline(), is(true));
    assertThat(configuration.attributes("storage").get("blobStoreName"), is("default"));
    assertThat(configuration.attributes("storage").get("writePolicy"), is("ALLOW_ONCE"));
    assertThat(configuration.attributes("cleanup").get("policyName"), is(Set.of("stale-objects")));
  }

  @Test
  void readsTheRequestBodyOfTheApi() throws Exception {
    GitLfsHostedRepositoryApiRequest request = new ObjectMapper().readValue("""
        {"name":"lfs","online":true,"format":"ignored","type":"ignored",
         "storage":{"blobStoreName":"default","strictContentTypeValidation":true,"writePolicy":"ALLOW"}}
        """, GitLfsHostedRepositoryApiRequest.class);

    assertThat(request.getFormat(), is("gitlfs"));
    assertThat(request.getType(), is("hosted"));
    assertThat(request.getStorage().getWritePolicy(), is("ALLOW"));
  }
}
