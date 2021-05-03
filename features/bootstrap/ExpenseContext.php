<?php

declare(strict_types = 1);

use Behat\Mink\Exception\ExpectationException;
use Behat\MinkExtension\Context\RawMinkContext;
use Firetrack\Tests\Traits\BrowserCapabilityDetectionTrait;

/**
 * Step definitions for interacting with expenses.
 */
class ExpenseContext extends RawMinkContext
{

    use BrowserCapabilityDetectionTrait;

    /**
     * Navigates to the form where an expense can be added.
     *
     * @Given I am on the add expense form
     */
    public function goToAddExpenseForm(): void
    {
        $this->visitPath('/expenses/add');

        // Check that the form was loaded correctly.
        if ($this->browserSupportsJavaScript()) {
            // JavaScript browsers cannot inspect the status code. Check for the
            // page title instead.
            if (!$this->getSession()->getPage()->has('xpath', '//h1[text()="Add expense"]')) {
                throw new ExpectationException(
                  'Could not find the title "Add expense" when going to the add expense form.',
                  $this->getSession()->getDriver()
                );
            }
        } else {
            $this->assertSession()->statusCodeEquals(200);
        }
    }

}
