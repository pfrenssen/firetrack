<?php

declare(strict_types = 1);

use Behat\Mink\Element\NodeElement;
use Behat\Mink\Exception\UnsupportedDriverActionException;
use Behat\MinkExtension\Context\RawMinkContext;
use Firetrack\Tests\Exception\ExpectationException;

/**
 * Step definitions for interacting with HTML pages.
 */
class HtmlContext extends RawMinkContext
{

    /**
     * Region CSS selectors keyed by human readable region names.
     *
     * @var array
     */
    protected array $regionMap;

    /**
     * Constructs a new HtmlContext object.
     *
     * @param array $region_map
     *   An associative array of CSS selectors that identify regions on the
     *   page, keyed by human readable region names.
     */
    public function __construct(array $region_map) {
        $this->regionMap = $region_map;
    }

    /**
     * Returns a region from the current page.
     *
     * @param string $region
     *   The human readable name of the region to return.
     *
     * @return \Behat\Mink\Element\NodeElement
     *   The region.
     *
     * @throws \Exception
     *   If the region cannot be found.
     */
    public function getRegion(string $region): NodeElement
    {
        $session = $this->getSession();
        if (!array_key_exists($region, $this->regionMap)) {
            throw new \Exception(sprintf('Region "%s" is not defined.', $region));
        }

        $element = $session->getPage()->find('css', $this->regionMap[$region]);
        if (!$element) {
            throw new \Exception(sprintf('No region "%s" found on the page %s.', $region, $session->getCurrentUrl()));
        }

        return $element;
    }

    /**
     * @Then I (should )see the heading :heading
     */
    public function assertHeading(string $heading): void
    {
        $element = $this->getSession()->getPage();
        foreach (['h1', 'h2', 'h3', 'h4', 'h5', 'h6'] as $tag) {
            $results = $element->findAll('css', $tag);
            foreach ($results as $result) {
                if ($result->getText() == $heading) {
                    return;
                }
            }
        }
        throw new \Exception(
            sprintf(
                "The text '%s' was not found in any heading on the page %s",
                $heading,
                $this->getSession()->getCurrentUrl()
            )
        );
    }

    /**
     * @Then I (should )not see the heading :heading
     */
    public function assertNoHeading(string $heading): void
    {
        $element = $this->getSession()->getPage();
        foreach (['h1', 'h2', 'h3', 'h4', 'h5', 'h6'] as $tag) {
            $results = $element->findAll('css', $tag);
            foreach ($results as $result) {
                if ($result->getText() == $heading) {
                    throw new \Exception(
                        sprintf(
                            "The text '%s' was found in a heading on the page %s",
                            $heading,
                            $this->getSession()->getCurrentUrl()
                        )
                    );
                }
            }
        }
    }

    /**
     * Clicks the link with the given id|title|alt|text.
     *
     * @When I click :link
     */
    public function clickLink(string $link): void
    {
        $this->getSession()->getPage()->clickLink($link);
    }

    /**
     * @Then I should see the link :link
     */
    public function assertLinkVisible($link)
    {
        $element = $this->getSession()->getPage();
        $result = $element->findLink($link);

        try {
            if ($result && !$result->isVisible()) {
                throw new \Exception(
                    sprintf("No link to '%s' on the page %s", $link, $this->getSession()->getCurrentUrl())
                );
            }
        } catch (UnsupportedDriverActionException $e) {
            // We catch the UnsupportedDriverActionException exception in case
            // this step is not being performed by a driver that supports javascript.
            // All other exceptions are valid.
        }

        if (empty($result)) {
            throw new \Exception(
                sprintf("No link to '%s' on the page %s", $link, $this->getSession()->getCurrentUrl())
            );
        }
    }

    /**
     * Links are not loaded on the page.
     *
     * @Then I should not see the link :link
     */
    public function assertNotLinkVisible($link)
    {
        $element = $this->getSession()->getPage();
        $result = $element->findLink($link);

        try {
            if ($result && $result->isVisible()) {
                throw new \Exception(
                    sprintf(
                        "The link '%s' was present on the page %s and was not supposed to be",
                        $link,
                        $this->getSession()->getCurrentUrl()
                    )
                );
            }
        } catch (UnsupportedDriverActionException $e) {
            // We catch the UnsupportedDriverActionException exception in case
            // this step is not being performed by a driver that supports javascript.
            // All other exceptions are valid.
        }

        if ($result) {
            throw new \Exception(
                sprintf(
                    "The link '%s' was present on the page %s and was not supposed to be",
                    $link,
                    $this->getSession()->getCurrentUrl()
                )
            );
        }
    }

    /**
     * Checks that a link is present in a given region on the page.
     *
     * @param string $link
     *   Link ID, title, text or image alt text.
     * @param string $region
     *   The human readable region name.
     *
     * @throws \Exception
     *   If region or link within it cannot be found.
     *
     * @Then I should see the link :link in the :region( region)
     */
    public function assertLinkRegion(string $link, string $region): void
    {
        $region_element = $this->getRegion($region);

        if (empty($region_element->findLink($link))) {
            throw new \Exception(sprintf('No link to "%s" in the "%s" region on the page %s', $link, $region, $this->getSession()->getCurrentUrl()));
        }
    }

    /**
     * Checks that a link is present in a given region on the page.
     *
     * @param string $link
     *   Link ID, title, text or image alt text.
     * @param string $region
     *   The human readable region name.
     *
     * @throws \Exception
     *   If region or link within it cannot be found.
     *
     * @Then I should not see the link :link in the :region( region)
     */
    public function assertNotLinkRegion(string $link, string $region): void
    {
        $region_element = $this->getRegion($region);

        if (!empty($region_element->findLink($link))) {
            throw new \Exception(sprintf('Link to "%s" in the "%s" region on the page %s', $link, $region, $this->getSession()->getCurrentUrl()));
        }
    }

    /**
     * Clicks a link in a given region on the page.
     *
     * @param string $link
     *   Link ID, title, text or image alt text.
     * @param string $region
     *   The human readable region name.
     *
     * @throws \Exception
     *   If region or link within it cannot be found.
     *
     * @When I click :link in the :region( region)
     */
    public function assertRegionLinkFollow(string $link, string $region): void
    {
        $region_element = $this->getRegion($region);

        $link_element = $region_element->findLink($link);
        if (empty($link_element)) {
            throw new \Exception(sprintf('The link "%s" was not found in the region "%s" on the page %s', $link, $region, $this->getSession()->getCurrentUrl()));
        }
        $link_element->click();
    }

    /**
     * Checks that the given field is empty.
     *
     * @param string $field
     *   The ID, name or label of the field to check.
     *
     * @throws \Exception
     *   If the field cannot be found.
     *
     * @Then the :field field should be empty
     * @Then the :field field should not contain a value
     */
    public function assertFieldEmpty(string $field): void
    {
        $field = $this->getSession()->getPage()->findField($field);
        if (!$field instanceof NodeElement) {
            throw new \Exception("Field '$field' not found.");
        }

        if (!empty($field->getValue())) {
            throw new ExpectationException("The '$field' field was expected to be empty but it was not.");
        }
    }

    /**
     * Checks that the given field contains the current date in YYYY-MM-DD format.
     *
     * @param string $field
     *   The ID, name or label of the field to check.
     *
     * @throws \Exception
     *   If the field cannot be found.
     *
     * @Then the :field field should contain today's date
     */
    public function assertFieldContainsCurrentDate(string $field): void
    {
        // Todo: Use the user's timezone.
        // Ref. https://github.com/pfrenssen/firetrack/issues/194
        $this->assertFieldContains($field, gmdate('Y-m-d'));
    }

    /**
     * Checks that the given field contains the given value.
     *
     * @param string $field
     *   The ID, name or label of the field to check.
     * @param string $expected_value
     *
     * @throws \Exception
     *   If the field cannot be found.
     */
    protected function assertFieldContains(string $field, string $expected_value): void
    {
        $element = $this->getSession()->getPage()->findField($field);
        if (!$element instanceof NodeElement) {
            throw new \Exception("Field '$field' not found.");
        }

        $actual_value = $element->getValue();
        if ($actual_value !== $expected_value) {
            throw new ExpectationException("The '$field' field was expected to contain '$expected_value' but it contained '$actual_value'.");
        }
    }
}
