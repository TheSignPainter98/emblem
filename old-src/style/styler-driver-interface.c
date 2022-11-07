#include "styler-driver-interface.h"

void pass_output_driver_data_to_styler(Styler* styler, OutputDriver* driver)
{
	int support			   = driver->support;
	styler->compose_styles = support & TS_CSS_STYLES_COMPOSED;
	styler->process_css	   = support & TS_CSS_STYLES;
	styler->process_scss   = styler->process_css || support & TS_CSS_UNPARSED;
}
