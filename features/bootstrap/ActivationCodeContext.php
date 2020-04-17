<?php

declare(strict_types = 1);

use Behat\Mink\Exception\ElementNotFoundException;
use Behat\MinkExtension\Context\RawMinkContext;
use Firetrack\Tests\Exception\ExpectationException;
use Firetrack\Tests\Traits\ActivationCodeTrait;
use Firetrack\Tests\Traits\MailgunTrait;

/**
 * Step definitions for interacting with activation codes.
 */
class ActivationCodeContext extends RawMinkContext
{

    use ActivationCodeTrait;
    use MailgunTrait;

    /**
     * Checks that an activation mail has been sent to the given email address.
     *
     * @param string $email
     *   The email address to which the activation mail has been sent.
     *
     * @Then an activation mail should have been sent to :email
     */
    public function assertActivationMessageSent(string $email): void
    {
        if (empty($this->getActivationMessage($email))) {
            throw new ExpectationException(sprintf('No activation message has been sent to %s', $email));
        }
    }

    /**
     * Enters the activation code found in the email sent to the given address in the field with the given label.
     *
     * @param string $field
     *   The label of the field in which to enter the activation code.
     * @param string $email
     *   The email address to which the activation code was sent.
     *
     * @When I fill in :field with the code that has been sent to :email
     */
    public function enterActivationCodeFromEmail(string $field, string $email): void
    {
        $message = $this->getActivationMessage($email);
        if (empty($message)) {
            throw new ExpectationException(sprintf('No activation code was sent to %s', $email));
        }

        $code = $this->getActivationCodeFromMessage($message);
        try {
            $this->getSession()->getPage()->fillField($field, $code);
        } catch (ElementNotFoundException $e) {
            throw new ExpectationException(
                sprintf(
                    'The field to fill in the activation code was not found on the page %s.',
                    $this->getSession()->getCurrentUrl()
                )
            );
        }
    }

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
