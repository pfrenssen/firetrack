<?php

declare(strict_types = 1);

use Behat\Mink\Exception\ElementNotFoundException;
use Behat\MinkExtension\Context\RawMinkContext;
use Firetrack\Tests\EmailMessage;
use Firetrack\Tests\Exception\ExpectationException;

/**
 * Step definitions for interacting with activation codes.
 */
class ActivationCodeContext extends RawMinkContext
{

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

        $code = $this->getActivationCode($message);
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
     * Returns the first activation message that was sent to the given email address.
     *
     * @param string $email
     *   The email address for which to return the message.
     *
     * @return EmailMessage|null
     *   The message, or NULL if no message was found.
     */
    protected function getActivationMessage(string $email): ?EmailMessage
    {
        foreach ($this->getMessagesTo($email) as $message) {
            if ($this->isActivationMessage($message)) {
                return $message;
            }
        }
        return null;
    }

    /**
     * Returns the activation code from the given activation mail.
     *
     * @param EmailMessage $message
     *   The message.
     *
     * @return string
     *   The activation code.
     *
     * @throws \InvalidArgumentException
     *   Thrown when the passed in message is not an activation mail.
     */
    protected function getActivationCode(EmailMessage $message): string
    {
        if (!$this->isActivationMessage($message)) {
            throw new \InvalidArgumentException('Can only retrieve activation codes from activation messages');
        }

        $matches = [];
        preg_match('/^Activation code: (\d{6})$/', $message->text, $matches);

        return $matches[1];
    }

    /**
     * Returns the messages which were sent to the given email address.
     *
     * @param string $email
     *
     * @return EmailMessage[]
     */
    protected function getMessagesTo(string $email): array
    {
        return array_filter(
            $this->getMessages(),
            function (EmailMessage $message) use ($email) {
                return $message->to === $email;
            }
        );
    }

    /**
     * Returns the list of email messages that have been sent through Mailgun.
     *
     * @return EmailMessage[]
     */
    protected function getMessages(): array
    {
        return array_map(
            function (string $line) {
                return EmailMessage::fromServerLogEntry($line);
            },
            $this->getLog()
        );
    }

    /**
     * Returns the contents of the Mailgun mock server log as an array of lines.
     *
     * @return string[]
     */
    protected function getLog(): array
    {
        // @todo Make the path to the log file configurable in behat.yml.
        $filename = getcwd() . '/mailgun-mock-server.log';
        return file($filename);
    }

    /**
     * Checks whether the passed in message is an activation message.
     *
     * @param EmailMessage $message
     *
     * @return bool
     */
    protected function isActivationMessage(EmailMessage $message): bool
    {
        return (bool) preg_match('/^Activation code: \d{6}$/', $message->text);
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
