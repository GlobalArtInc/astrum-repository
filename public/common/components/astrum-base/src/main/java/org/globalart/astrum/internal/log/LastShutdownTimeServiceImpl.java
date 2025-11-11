
package org.globalart.astrum.internal.log;

import java.io.File;
import java.io.IOException;
import java.text.DateFormat;
import java.text.ParseException;
import java.text.SimpleDateFormat;
import java.util.Date;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.log.LastShutdownTimeService;
import org.globalart.astrum.common.log.LogManager;

import com.google.common.annotations.VisibleForTesting;
import org.apache.commons.io.input.ReversedLinesFileReader;
import org.springframework.beans.factory.annotation.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.common.log.LogManager.DEFAULT_LOGGER;
import org.springframework.stereotype.Component;

/**
 * {@link LastShutdownTimeService} implementation.
 *
 * @since 3.13
 */
@Component
public class LastShutdownTimeServiceImpl
    implements LastShutdownTimeService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String ASTRUM_LOG_PATTERN = "(?<time>\\d{4}-\\d{2}-\\d{2} \\d{2}:\\d{2}:\\d{2})";

  private static final String ASTRUM_DATE_FORMAT = "yyyy-MM-dd HH:mm:ss";

  private static final String GROUP_NAME = "time";

  private static final String START_INDICATOR = "org.globalart.astrum.logging.AstrumLogActivator - start";

  private static final int DEFAULT_LINE_READING_LIMIT = 10_000;

  private final DateFormat astrumFormat = new SimpleDateFormat(ASTRUM_DATE_FORMAT);

  private final Pattern astrumPattern = Pattern.compile(ASTRUM_LOG_PATTERN);

  private final LogManager logManager;

  private Optional<Date> shutdownTimeGuess = null;

  @Autowired
  public LastShutdownTimeServiceImpl(
      final LogManager logManager,
      @Value("${astrum.log.lastShutdownTime.enabled:true}") final boolean enabled)
  {
    this.logManager = logManager;
    if (!enabled) {
      this.shutdownTimeGuess = Optional.empty();
    }
  }

  @Override
  public synchronized Optional<Date> estimateLastShutdownTime() {
    if (shutdownTimeGuess == null) {
      shutdownTimeGuess = findBestEstimate(logManager);
    }
    return shutdownTimeGuess;
  }

  private Optional<Date> findBestEstimate(final LogManager logManager) {
    File astrumFile = logManager.getLogFileForLogger(DEFAULT_LOGGER).orElse(null);
    Optional<Date> estimatedTime = Optional.empty();

    if (astrumFile == null) {
      log.warn("Missing log file for {} , so last shutdown time can't be estimated.", DEFAULT_LOGGER);
    }
    else if (astrumFile.length() == 0) {
      log.warn("Empty log file {} , so last shutdown time can't be estimated.", astrumFile);
    }
    else {
      try (ReversedLinesFileReader logReader = new ReversedLinesFileReader(astrumFile)) {
        estimatedTime = findShutdownTimeInLog(logReader, START_INDICATOR, astrumPattern, DEFAULT_LINE_READING_LIMIT,
            GROUP_NAME, astrumFormat);
      }
      catch (Exception e) {
        log.warn("Failed to process file {}.  Assuming no previous start time", astrumFile, e);
      }
    }

    return estimatedTime;
  }

  @VisibleForTesting
  Optional<Date> findShutdownTimeInLog(
      final ReversedLinesFileReader logReader,
      final String startIndicator,
      final Pattern timestampPattern,
      final int lineScanLimit,
      final String groupName,
      final DateFormat dateFormat) throws IOException, ParseException
  {

    Optional<Integer> linesRead = advanceReaderToShutdownLine(logReader, startIndicator, lineScanLimit);
    if (linesRead.isPresent()) {
      return findTimeFromPreviousInstance(logReader, timestampPattern, lineScanLimit - linesRead.get(), groupName,
          dateFormat);
    }
    else {
      return Optional.empty();
    }
  }

  /**
   * The main work of this method is done in moving the logReader to the point just before the last shutdown
   */
  private Optional<Integer> advanceReaderToShutdownLine(
      final ReversedLinesFileReader logReader,
      final String startIndicator,
      final int lineScanLimit) throws IOException
  {
    int lineCount = 0;
    String line;
    while (lineCount++ < lineScanLimit && (line = logReader.readLine()) != null) {
      if (line.contains(startIndicator)) {
        return Optional.of(lineCount);
      }
    }

    return Optional.empty();
  }

  private Optional<Date> findTimeFromPreviousInstance(
      final ReversedLinesFileReader logReader,
      final Pattern timestampPattern,
      final int lineScanLimit,
      final String groupName,
      final DateFormat dateFormat) throws IOException, ParseException
  {
    int lineCount = 0;
    String line;
    while (lineCount++ < lineScanLimit && (line = logReader.readLine()) != null) {
      Matcher matcher = timestampPattern.matcher(line);
      if (matcher.find()) {
        return Optional.of(dateFormat.parse(matcher.group(groupName)));
      }
    }
    return Optional.empty();
  }
}
