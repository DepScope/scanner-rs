"""
Python Packages Analysis Page
"""
import streamlit as st
import pandas as pd
import plotly.express as px

st.set_page_config(
    page_title="Python Packages",
    page_icon="🐍",
    layout="wide"
)

def load_data(csv_path: str) -> pd.DataFrame:
    """Load and prepare the CSV data"""
    try:
        df = pd.read_csv(csv_path)
        df.columns = df.columns.str.strip()

        # Normalize column names
        if 'package_name' in df.columns:
            df['package'] = df['package_name']
        if 'has_version' in df.columns:
            df['version'] = df['has_version']
        if 'should_path' in df.columns and 'location' not in df.columns:
            df['location'] = df['should_path']
        elif 'application_root' in df.columns and 'location' not in df.columns:
            df['location'] = df['application_root']

        if 'parent_package' in df.columns:
            df['match_package'] = df['parent_package'].fillna('none')
        elif 'match_package' not in df.columns:
            df['match_package'] = 'none'

        return df
    except FileNotFoundError:
        st.error(f"CSV file not found: {csv_path}")
        st.stop()
    except Exception as e:
        st.error(f"Error loading CSV: {e}")
        st.stop()

def main():
    st.title("🐍 Python Packages Analysis")

    # Load data from uploaded file or path
    if 'uploaded_df' in st.session_state and st.session_state.uploaded_df is not None:
        df = st.session_state.uploaded_df.copy()
    else:
        csv_path = st.session_state.get('csv_path', '../output.csv')
        df = load_data(csv_path)

    # Normalize columns if needed
    if 'package_name' in df.columns and 'package' not in df.columns:
        df['package'] = df['package_name']
    if 'has_version' in df.columns and 'version' not in df.columns:
        df['version'] = df['has_version']
    if 'should_path' in df.columns and 'location' not in df.columns:
        df['location'] = df['should_path']

    # Filter for Python packages using ecosystem column if available
    if 'ecosystem' in df.columns:
        python_df = df[df['ecosystem'].str.lower() == 'python'].copy()
    else:
        python_df = pd.DataFrame()

    # If filtering resulted in empty dataframe, show message
    if len(python_df) == 0:
        st.info("No Python packages found in the dataset.")
        python_df = pd.DataFrame()

    # Overview metrics
    st.header("📊 Python Overview")
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Total Python Packages", len(python_df))

    with col2:
        unique_packages = python_df['package'].nunique() if 'package' in python_df.columns else 0
        st.metric("Unique Packages", unique_packages)

    with col3:
        unique_locations = python_df['location'].nunique() if 'location' in python_df.columns else 0
        st.metric("Locations", unique_locations)

    with col4:
        if 'match_package' in python_df.columns:
            infected = python_df[python_df['match_package'].str.lower() != 'none'].shape[0]
        else:
            infected = 0
        st.metric("⚠️ Infected", infected, delta_color="inverse")

    # Top 20 most used Python packages
    st.header("🔝 Top 20 Most Used Python Packages")

    if 'package' in python_df.columns and len(python_df) > 0:
        package_counts = python_df['package'].value_counts().head(20)

        fig = px.bar(
            x=package_counts.values,
            y=package_counts.index,
            orientation='h',
            labels={'x': 'Occurrences', 'y': 'Package Name'},
            title="Top 20 Python Packages by Frequency",
            color=package_counts.values,
            color_continuous_scale='Greens'
        )
        fig.update_layout(height=700, showlegend=False)
        st.plotly_chart(fig, width='stretch')

        # Show table with details
        st.subheader("Package Details")
        top_packages = package_counts.head(20).index.tolist()

        # Group by package and show versions
        for package in top_packages[:5]:  # Show details for top 5
            with st.expander(f"🐍 {package} ({package_counts[package]} occurrences)"):
                pkg_df = python_df[python_df['package'] == package]
                if 'version' in pkg_df.columns:
                    version_counts = pkg_df['version'].value_counts()
                    col1, col2 = st.columns(2)

                    with col1:
                        st.write("**Versions:**")
                        for version, count in version_counts.items():
                            st.write(f"- `{version}`: {count} locations")

                    with col2:
                        st.write("**Sample Locations:**")
                        for loc in pkg_df['location'].head(3):
                            st.write(f"- {loc}")
    else:
        st.info("No Python packages found in the dataset")

    # Version consistency analysis
    st.header("📊 Version Consistency")

    if 'package' in python_df.columns and 'version' in python_df.columns and len(python_df) > 0:
        # Find packages with multiple versions
        version_diversity = python_df.groupby('package')['version'].nunique().sort_values(ascending=False)
        inconsistent = version_diversity[version_diversity > 1].head(10)

        if len(inconsistent) > 0:
            st.warning(f"Found {len(version_diversity[version_diversity > 1])} packages with multiple versions")

            fig = px.bar(
                x=inconsistent.values,
                y=inconsistent.index,
                orientation='h',
                labels={'x': 'Number of Different Versions', 'y': 'Package'},
                title="Top 10 Packages with Most Version Variations",
                color=inconsistent.values,
                color_continuous_scale='Oranges'
            )
            fig.update_layout(height=400)
            st.plotly_chart(fig, width='stretch')
        else:
            st.success("✅ All packages have consistent versions!")

    # Common Python frameworks detection
    st.header("🔧 Framework Detection")

    if 'package' in python_df.columns and len(python_df) > 0:
        frameworks = {
            'Django': ['django'],
            'Flask': ['flask'],
            'FastAPI': ['fastapi'],
            'Pandas': ['pandas'],
            'NumPy': ['numpy'],
            'Requests': ['requests'],
            'SQLAlchemy': ['sqlalchemy'],
            'Pytest': ['pytest']
        }

        detected = {}
        for framework, packages in frameworks.items():
            for pkg in packages:
                if pkg in python_df['package'].str.lower().values:
                    detected[framework] = len(python_df[python_df['package'].str.lower() == pkg])

        if detected:
            col1, col2 = st.columns(2)
            with col1:
                st.write("**Detected Frameworks:**")
                for framework, count in detected.items():
                    st.write(f"- {framework}: {count} occurrences")

    # Raw data
    with st.expander("🔍 View Python Package Data"):
        st.dataframe(python_df, width='stretch', height=400)

if __name__ == "__main__":
    main()
