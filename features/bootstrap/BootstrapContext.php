<?php

use Behat\MinkExtension\Context\RawMinkContext;

/**
 * Step definitions for interacting with Bootstrap components.
 */
class BootstrapContext extends RawMinkContext
{

    /**
     * Checks that the given invalid feedback message is present on the page.
     *
     * @param string $feedback
     *   The feedback message.
     *
     * @throws Exception
     *   Thrown when the feedback message is not found.
     *
     * @see https://getbootstrap.com/docs/4.0/components/forms/#server-side
     *
     * @Then I should see the invalid feedback message :feedback
     */
    public function assertInvalidFeedback(string $feedback): void
    {
        $xpath = '//*[*[contains(concat(" ", @class, " "), " form-control ") and contains(concat(" ", @class, " "), " is-invalid ")]]/*[contains(concat(" ", @class, " "), " invalid-feedback ") and text() = "' . $feedback . '"]';
        if (empty($this->getSession()->getPage()->find('xpath', $xpath))) {
            throw new Exception(sprintf("The invalid feedback message '%s' was not found on the page %s.", $feedback, $this->getSession()->getCurrentUrl()));
        }
    }

    /**
     * Checks that the given invalid feedback message is not present on the page.
     *
     * @param string $feedback
     *   The feedback message.
     *
     * @throws Exception
     *   Thrown when the feedback message is found.
     *
     * @see https://getbootstrap.com/docs/4.0/components/forms/#server-side
     *
     * @Then I should not see the invalid feedback message :feedback
     */
    public function assertNoInvalidFeedback(string $feedback): void
    {
        $xpath = '//*[*[contains(concat(" ", @class, " "), " form-control ") and contains(concat(" ", @class, " "), " is-invalid ")]]/*[contains(concat(" ", @class, " "), " invalid-feedback ") and text() = "' . $feedback . '"]';
        if (!empty($this->getSession()->getPage()->find('xpath', $xpath))) {
            throw new Exception(sprintf("The invalid feedback message '%s' was found on the page %s but was not expected to be.", $feedback, $this->getSession()->getCurrentUrl()));
        }
    }

}
