<?php

declare(strict_types = 1);

use Behat\MinkExtension\Context\RawMinkContext;
use Firetrack\Tests\Exception\ExpectationException;

/**
 * Step definitions for interacting with Bootstrap components.
 */
class BootstrapContext extends RawMinkContext
{

    /**
     * Checks that the given invalid feedback message is present on the page.
     *
     * @param string $message
     *   The feedback message.
     *
     * @see https://getbootstrap.com/docs/4.0/components/forms/#server-side
     *
     * @Then I should see the form validation message :message
     */
    public function assertFormValidationMessage(string $message): void
    {
        // XPath equivalent of the Bootstrap CSS selector (`.form-control.is-invalid ~ .invalid-feedback`) that makes
        // the invalid feedback appear, so we can match on the text value.
        $xpath = '//*[contains(concat(" ", @class, " "), " invalid-feedback ") and ../*[contains(concat(" ", @class, " "), " form-control ")] and ../*[contains(concat(" ", @class, " "), " is-invalid ")] and text() = "' . $message . '"]';
        if (empty($this->getSession()->getPage()->find('xpath', $xpath))) {
            throw new ExpectationException(sprintf("The form validation message '%s' was not found on the page %s.", $message, $this->getSession()->getCurrentUrl()));
        }
    }

    /**
     * Checks that the given invalid feedback message is not present on the page.
     *
     * @param string $message
     *   The feedback message.
     *
     * @see https://getbootstrap.com/docs/4.0/components/forms/#server-side
     *
     * @Then I should not see the form validation message :message
     */
    public function assertNoFormValidationMessage(string $message): void
    {
        try {
            $this->assertFormValidationMessage($message);
            throw new ExpectationException(sprintf("The form validation message '%s' was found on the page %s but was not expected to be.", $message, $this->getSession()->getCurrentUrl()));
        } catch (ExpectationException $e) {}
    }

}
