<?php

declare(strict_types = 1);

use Behat\MinkExtension\Context\RawMinkContext;

/**
 * Step definitions for interacting with Mailgun messages.
 */
class MailgunContext extends RawMinkContext
{

    /**
     * Purges the log file of the Mailgun mock server before starting a scenario.
     *
     * This makes sure that scenarios that access the Mailgun log will not accidentally read data from previous tests.
     *
     * @beforeScenario
     */
    public function purgeMailgunLog(): void
    {
        // @todo Make the path to the log file configurable in behat.yml.
        $filename = getcwd() . '/mailgun-mock-server.log';
        if (is_file($filename)) {
            unlink($filename);
        }
    }

}
