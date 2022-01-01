<?php

declare(strict_types = 1);

use Behat\Mink\Exception\ExpectationException;
use Behat\MinkExtension\Context\RawMinkContext;
use Firetrack\Tests\Traits\BrowserCapabilityDetectionTrait;
use Firetrack\Tests\Traits\FiretrackCliTrait;

/**
 * Step definitions for interacting with expenses.
 */
class ExpenseContext extends RawMinkContext
{

    use BrowserCapabilityDetectionTrait;
    use FiretrackCliTrait;

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

    /**
     * @Then the user :email should have :count expense(s)
     */
    public function assertExpenseCount(int $count, string $email): void
    {
        $actual_count = $this->getExpenseCount($email);
        if ($count !== $actual_count) {
            throw new ExpectationException(
                sprintf('Expected %d expenses for the user %s, found %d expenses.', $count, $email, $actual_count),
                $this->getSession()->getDriver()
            );
        }
    }

    /**
     * Returns the number of expenses, optionally filtered by user.
     *
     * @param string|null $email
     *
     * @return string
     *   The activation code.
     */
    protected function getExpenseCount(?string $email): int
    {
        assert(
            in_array(FiretrackCliTrait::class, class_uses($this)),
            __METHOD__ . ' depends on FiretrackCliTrait. Please include it in ' . __CLASS__
        );

        $result = $this->executeCommand('expense list' . ($email ? ' ' . escapeshellarg($email) : '') . ' --count');
        return (int) reset($result);
    }
}
