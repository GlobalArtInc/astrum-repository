
package org.globalart.astrum.coreui.internal.wonderland;

import java.io.IOException;

import jakarta.ws.rs.core.Response;

import org.globalart.astrum.common.wonderland.AuthTicketService;
import org.globalart.astrum.common.wonderland.DownloadService;
import org.globalart.astrum.common.wonderland.DownloadService.Download;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.Mock;

import static com.google.common.net.HttpHeaders.CONTENT_DISPOSITION;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class DownloadResourceTest
{
  @Mock
  private DownloadService downloadService;

  @Mock
  private AuthTicketService authTicketService;

  /**
   * Fix for ASTRUM-40992
   */
  @Test
  void downloadZipUsesCorrectFileNameHeader() {
    DownloadResource underTest = new DownloadResource(downloadService, authTicketService);
    String fileName = "supportZip-timestamp.zip";
    mockAuthenticatedDownload(fileName);

    Response response = underTest.downloadZip(fileName);

    assertThat(response.getHeaderString(CONTENT_DISPOSITION), is("attachment; filename=\"" + fileName + "\""));
  }

  private void mockAuthenticatedDownload(final String fileName) {
    Download mockDownload = mock(Download.class);
    String fileAuthTicket = fileName + "-authTicket";
    when(authTicketService.createTicket()).thenReturn(fileAuthTicket);
    try {
      when(downloadService.get(fileName, fileAuthTicket)).thenReturn(mockDownload);
    }
    catch (IOException e) {
      // swallow exception that will never happen
    }
  }

}
