package org.globalart.astrum.repository.rest.internal.api;

import java.util.HashMap;
import java.util.Map;

import org.junit.jupiter.api.Test;

import static org.globalart.astrum.repository.rest.internal.api.AuthorizingRepositoryManagerImpl.keepWriteOnlySecrets;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.Matchers.sameInstance;

class AuthorizingRepositoryManagerWriteOnlySecretsTest
{
  @Test
  void keepsTheSavedPassphraseWhenAnUpdateLeavesItOut() {
    Map<String, Map<String, Object>> result =
        keepWriteOnlySecrets(signing("KEY", "secret"), signing("KEY", null));

    assertThat(result.get("terraformSigning").get("passphrase"), is("secret"));
    assertThat(result.get("terraformSigning").get("keypair"), is("KEY"));
  }

  @Test
  void keepsTheSavedPassphraseWhenAnUpdateSendsAnEmptyOne() {
    Map<String, Map<String, Object>> result =
        keepWriteOnlySecrets(signing("KEY", "secret"), signing("KEY", ""));

    assertThat(result.get("terraformSigning").get("passphrase"), is("secret"));
  }

  @Test
  void replacesThePassphraseWhenAnUpdateSendsANewOne() {
    Map<String, Map<String, Object>> requested = signing("KEY", "new");

    assertThat(keepWriteOnlySecrets(signing("KEY", "secret"), requested), sameInstance(requested));
  }

  @Test
  void dropsThePassphraseWithANewSigningKey() {
    Map<String, Map<String, Object>> result =
        keepWriteOnlySecrets(signing("KEY", "secret"), signing("OTHER KEY", null));

    assertThat(result.get("terraformSigning").get("passphrase"), nullValue());
  }

  @Test
  void leavesAttributesWithoutSigningAlone() {
    Map<String, Map<String, Object>> requested = new HashMap<>();
    requested.put("storage", new HashMap<>(Map.of("blobStoreName", "default")));

    assertThat(keepWriteOnlySecrets(signing("KEY", "secret"), requested), sameInstance(requested));
    assertThat(keepWriteOnlySecrets(null, requested), sameInstance(requested));
    assertThat(keepWriteOnlySecrets(requested, null), nullValue());
  }

  @Test
  void leavesSigningWithoutASavedPassphraseAlone() {
    Map<String, Map<String, Object>> requested = signing("KEY", null);

    assertThat(keepWriteOnlySecrets(signing("KEY", null), requested), sameInstance(requested));
  }

  private static Map<String, Map<String, Object>> signing(final String keypair, final String passphrase) {
    Map<String, Object> signing = new HashMap<>();
    signing.put("keypair", keypair);
    if (passphrase != null) {
      signing.put("passphrase", passphrase);
    }
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    attributes.put("terraformSigning", signing);
    attributes.put("storage", new HashMap<>(Map.of("blobStoreName", "default")));
    return attributes;
  }
}
