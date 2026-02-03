
package org.globalart.astrum.repository.view;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;

@SuppressWarnings("resource")
@ExtendWith(MockitoExtension.class)
class ContentTest
{
  @Mock
  Payload payload;

  @Test
  void testHasRedirect() {
    Content underTest = new Content(payload);
    assertThat(underTest.hasRedirect(), is(false));

    when(payload.hasRedirect()).thenReturn(true);
    assertThat(underTest.hasRedirect(), is(true));

    underTest.setDisableRedirect();
    assertThat(underTest.hasRedirect(), is(false));

  }
}
