
package org.globalart.astrum.repository.rest.internal.resources;

import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.globalart.astrum.repository.search.SearchService;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;

import org.apache.shiro.authz.UnauthenticatedException;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

@ExtendWith({MockitoExtension.class, AuthenticationExtension.class})
class SuggestResourceTest
{
  @Mock
  SearchService searchService;

  @InjectMocks
  SuggestResource underTest;

  @Test
  public void testSuggest_unauthenticated() {
    assertThrows(UnauthenticatedException.class, () -> underTest.suggest(null, null, 50));
  }

  @Test
  public void isRegisteredWithoutThePreviewUiProperty() {
    assertNull(SuggestResource.class.getAnnotation(ConditionalOnProperty.class));
  }
}
