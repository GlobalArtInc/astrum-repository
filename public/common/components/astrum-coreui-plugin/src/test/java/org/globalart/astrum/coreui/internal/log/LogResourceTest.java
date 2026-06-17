
package org.globalart.astrum.coreui.internal.log;

import org.globalart.astrum.common.log.LogMarker;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.InjectMocks;
import org.mockito.Mock;

import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class LogResourceTest
{
  @Mock
  private LogMarker logMarker;

  @InjectMocks
  private LogResource underTest;

  @Test
  void markWithNullMessageUsesDefault() {
    underTest.mark(null);
    verify(logMarker).markLog(LogResource.DEFAULT_MARK);
  }

  @Test
  void markWithEmptyMessageUsesDefault() {
    underTest.mark("");
    verify(logMarker).markLog(LogResource.DEFAULT_MARK);
  }

  @Test
  void markWithCustomMessage() {
    String customMessage = "custom log mark";
    underTest.mark(customMessage);
    verify(logMarker).markLog(customMessage);
  }
}
